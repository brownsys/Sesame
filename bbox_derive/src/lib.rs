extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod render;

#[proc_macro_derive(BBoxRender)]
pub fn derive_boxed_serialize(input: TokenStream) -> TokenStream {
  let input = parse_macro_input!(input as DeriveInput);
  render::derive_boxed_serialize_impl(input).into()
}


