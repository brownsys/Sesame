extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Data, DataStruct, Fields, Field, GenericParam};
use syn::token::{Comma};
use syn::punctuated::{Punctuated};

// Make sure you import these in client code.
// use std::collections::BTreeMap;
// use bbox::{ValueOrBBox, BBoxRender};

pub fn derive_boxed_serialize_impl(input: DeriveInput) -> TokenStream {
  // struct name we are deriving for.
  let input_name = input.ident;

  // fields inside struct.
  let fields: Punctuated<Field, Comma> = match input.data {
    Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => fields.named,
    _ => panic!("this derive macro only works on structs with named fields"),
  };

  let puts = fields.into_iter().map(|field| {
    let field = field.ident.unwrap();
    let field_name: String = field.to_string();

    quote! {
      map.insert(String::from(#field_name), self.#field.render());
    }
  });
  
  // Generics if any.
  let generics = input.generics;
  let where_clause = &generics.where_clause;
  let template = generics.lt_token.map(|_| {
    let params = generics.params.iter().map(|generic_param| {
      match generic_param {
        GenericParam::Lifetime(param) => {
          let param = &param.lifetime;
          quote! { #param }
        },
        GenericParam::Type(param) => {
          let param = &param.ident;
          quote! { #param }
        },
        GenericParam::Const(param) => {
          let param = &param.ident;
          quote! { #param }
        },
      }
    });
    quote! { < #(#params),* > }
  });

  // Impl trait.
  quote! {
    #[automatically_derived]
    impl #generics bbox::BBoxRender for #input_name #template #where_clause {
      fn render<'a>(&'a self) -> bbox::Renderable<'a> {
        let mut map: std::collections::BTreeMap<String, bbox::Renderable<'a>> = std::collections::BTreeMap::new();
        #(#puts)*
        bbox::Renderable::Dict(map)
      }
    }
  }
}
