extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemStruct, Lit, Token};
use syn::punctuated::Punctuated;
use syn::parse::Parser;

mod render;
mod policy;

#[proc_macro_derive(BBoxRender)]
pub fn derive_boxed_serialize(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  render::derive_boxed_serialize_impl(input).into()
}

#[proc_macro_attribute]
pub fn schema_policy(args: TokenStream, input: TokenStream) -> TokenStream {
  let mut result = input.clone();
  let args = parse_macro_input!(args as policy::SchemaPolicyArgs);
  let parsed = parse_macro_input!(input as ItemStruct);
  let additional: TokenStream = policy::schema_policy_impl(args, parsed).into();
  result.extend(additional.into_iter());
  result
}
