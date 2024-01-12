extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Data, DataStruct, DeriveInput, Field, Fields, Visibility, Type};
use attribute_derive::FromAttr;

#[derive(FromAttr)]
#[attribute(ident = magic_unbox_out)]
struct MagicUnboxArgs {
  #[attribute(optional = false)]
  name: String,
  to_derive: Option<Vec<Ident>>, 
}

pub fn derive_magic_unbox_impl(input: DeriveInput) -> TokenStream { 
    // struct name we are deriving for.
    let input_name: Ident = input.ident;

    let input_vis: Visibility = input.vis;

    let out_attrs: MagicUnboxArgs = MagicUnboxArgs::from_attributes(&input.attrs).unwrap();
    let derived_name: Ident = syn::Ident::new(out_attrs.name.as_str(), input_name.span());
    let trait_vec: Vec<Ident> = out_attrs.to_derive.clone().unwrap_or(vec![]); 
    
    let iter_traits = trait_vec.clone()
                                                                  .into_iter()
                                                                  .map(|trait_ident| {
                                                                  quote!{ #trait_ident }});
    let derive_traits = { 
      if trait_vec.len() > 0 {
        quote!{ #[derive(#(#iter_traits),*)] } 
      } else {
        quote!{}
      }
    };  

    // get fields inside struct.
    let fields: Punctuated<Field, Comma> = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };
    
    // Copy over struct fields but with types as MagicUnbox
    let build_struct_fields = fields.clone().into_iter().map(|field| {
      let field_vis = field.vis; 
      let field_ident = field.ident.clone().unwrap();
      let field_type = field.ty;
      quote! { 
        #field_vis #field_ident: <#field_type as MagicUnbox>::Out
      }
    }); 

    // Create map of struct fields to MagicUnboxEnums
    let puts_to_enum = fields.clone().into_iter().map(|field| {
        let field_ident = field.ident.unwrap();
        let field_name: String = field_ident.to_string();
        quote! { //map is HashMap defined in to_enum
          map.insert(::std::string::String::from(#field_name), self.#field_ident.to_enum());
        }
    });

    //pop the fields into the new struct 
     let gets_from_enum = fields.clone().into_iter().map(|field| {
      let field_ident: Ident = field.ident.unwrap();
      let field_name: String = field_ident.to_string();
      let field_type: Type = field.ty;
      quote! { 
        #field_ident: <#field_type as MagicUnbox>::from_enum(hashmap.remove(#field_name).unwrap())?,
      }
    }); 

    // Generics if any.
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Impl trait.
    quote! {
      #[automatically_derived]
      
      #derive_traits
      #input_vis struct #derived_name { 
        #(#build_struct_fields,)*
      } 

      impl #impl_generics ::bbox::bbox::MagicUnbox for #input_name #ty_generics #where_clause {
        type Out = #derived_name; 

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
