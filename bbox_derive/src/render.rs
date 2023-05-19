extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Data, DataStruct, Fields, Field};
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

  // Impl trait.
  quote! {
    #[automatically_derived]
    impl BBoxRender for #input_name {
      fn render<'a>(&'a self) -> ValueOrBBox<'a> {
        let mut map: BTreeMap<String, ValueOrBBox<'a>> = BTreeMap::new();
        #(#puts)*
        ValueOrBBox::Dict(map)
      }
    }
  }
}



/*
pub trait BBoxRender {
  fn render<'a>(&'a self) -> ValueOrBBox<'a>;
}
*/
