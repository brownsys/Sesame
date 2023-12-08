extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Data, DataStruct, DeriveInput, Field, Fields};
use std::collections::HashMap;

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

    let mut field_type_map = HashMap::new();
    
    for field in fields.clone(){
      let field_name: String = field.ident.unwrap().to_string();
      let field_type = field.ty;
      field_type_map.insert(field_name, field_type.clone());
    } 

    let puts_to_enum = fields.clone().into_iter().map(|field| {
        let field = field.ident.unwrap();
        let field_name: String = field.to_string();
        quote! {
          map.insert(::std::string::String::from(#field_name), self.#field.to_enum());
        }
    });

    let field_for_struct = fields.clone().into_iter().map(|field| {
      let field = field.ident.unwrap();
      let field_name: String = field.to_string();
      let field_type =  field_type_map.get(&field_name);
      quote! { //trying to pop the fields into the new struct
        #field_name: <#field_type as MagicUnbox>::from_enum(hashmap.remove(#field_name).unwrap())?,
      }
    });

    let lite_struct_name = syn::Ident::new(&format!("{}Lite", input_name), input_name.span());

    let new_struct = quote! {
      struct #lite_struct_name {
          #fields,
      }
    };

    // Generics if any.
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Impl trait.
    quote! {
      #[automatically_derived]
      impl #impl_generics ::bbox::bbox::MagicUnbox for #input_name #ty_generics #where_clause {

        type Out = #lite_struct_name;

        fn to_enum(self) -> ::bbox::bbox::MagicUnboxEnum {
          let mut map: ::std::collections::HashMap<::std::string::String, ::bbox::bbox::MagicUnboxEnum> = ::std::collections::HashMap::new();
          #(#puts_to_enum)*
          ::bbox::bbox::MagicUnbox::Struct(map)
        }

        fn from_enum(e: MagicUnboxEnum) -> Result<Self::Out, ()> {
          match e {
            MagicUnboxEnum::Struct(mut hashmap) => Ok(Self::Out {
                #(#field_for_struct)*
            }),
            _ => Err(()),
          }
        }
      }
    }
}

