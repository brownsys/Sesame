extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields};
use std::collections::HashMap;
use syn::{GenericArgument, Path, PathArguments, PathSegment};


fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
  match *ty {
      syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
      _ => None,
  }
}

fn extract_bboxed_segment(path: &Path) -> Option<&PathSegment> {
  let idents_of_path = path
      .segments
      .iter()
      .into_iter()
      .fold(String::new(), |mut acc, v| {
          acc.push_str(&v.ident.to_string());
          acc.push('|');
          acc
      });
  vec!["BBox|", "bbox|BBox|"]   //possible ways to write BBox
      .into_iter()
      .find(|s| &idents_of_path == *s)
      .and_then(|_| path.segments.last())
}

fn extract_type_from_bbox(ty: &syn::Type) -> Option<&syn::Type> {
    extract_type_path(ty)
      .and_then(|path| extract_bboxed_segment(path))
      .and_then(|path_seg| {
          let type_params = &path_seg.arguments;
          match *type_params {
              PathArguments::AngleBracketed(ref params) => params.args.first(),
              _ => None,
          }
      })
      .and_then(|generic_arg| match *generic_arg {
          GenericArgument::Type(ref ty) => Some(ty),
          _ => None,
      })
}

pub fn derive_magic_unbox_impl(input: DeriveInput) -> TokenStream { 
    // struct name we are deriving for.
    let input_name = input.ident;

    // get fields inside struct.
    let fields: Punctuated<Field, Comma> = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };

    let derived_name: Ident = syn::Ident::new(&format!("{}Lite", input_name), input_name.span());

    let build_struct = fields.clone().into_iter().map(|field| {
      let field_ident = field.ident.clone().unwrap();
      let field_type = field.ty;
      let field_type2 = field_type.clone();
      let unboxed_field_type = match extract_type_from_bbox(&field_type) {
          None => field_type,
          Some(ty) => ty.clone(),
      }; 
      quote! { // TODO test mechanics of unboxing types
        pub #field_ident: <#field_type2 as MagicUnbox>::Out
      }
    }); 

    let puts_to_enum = fields.clone().into_iter().map(|field| {
        let field_ident = field.ident.unwrap();
        let field_name: String = field_ident.to_string();
        quote! { //map is HashMap defined in to_enum
          map.insert(::std::string::String::from(#field_name), self.#field_ident.to_enum());
        }
    });

     let gets_from_enum = fields.clone().into_iter().map(|field| {
      let field_ident = field.ident.unwrap();
      let field_name: String = field_ident.to_string();
      let field_type = field.ty;
      quote! { //pop the fields into the new struct 
        #field_ident: <#field_type as MagicUnbox>::from_enum(hashmap.remove(#field_name).unwrap())?,
      }
    }); 

    // Generics if any.
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Impl trait.
    quote! {
      #[automatically_derived]

      
      // #[derive(BBoxRender, Clone, Serialize)] // TODO less rigid option?
      pub struct #derived_name {
        #(#build_struct,)*
      } 

      impl #impl_generics ::bbox::bbox::MagicUnbox for #input_name #ty_generics #where_clause {
        type Out = #derived_name; 
        //type Out = #input_name;

        fn to_enum(self) -> ::bbox::bbox::MagicUnboxEnum {
          let mut map: ::std::collections::HashMap<::std::string::String, ::bbox::bbox::MagicUnboxEnum> = ::std::collections::HashMap::new();
          #(#puts_to_enum)*
          ::bbox::bbox::MagicUnboxEnum::Struct(map)
        }

        fn from_enum(e: MagicUnboxEnum) -> Result<Self::Out, ()> {
          match e {
            MagicUnboxEnum::Struct(mut hashmap) => Ok(Self::Out {
                #(#gets_from_enum)* 
            }),
            _ => Err(()),
          }
        }
      }
    }
  }
