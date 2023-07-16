extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, ItemFn, ItemStruct};

mod form;
mod policy;
mod render;
mod route;

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

#[proc_macro_derive(FromBBoxForm)]
pub fn derive_from_bbox_form(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    form::derive_from_bbox_form_impl(input).into()
}

#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut result = input.clone();
    let args = parse_macro_input!(args as route::RouteArgs<route::Unknown>);
    let parsed = parse_macro_input!(input as ItemFn);
    let additional: TokenStream = route::route_impl(args, parsed).into();
    result.extend(additional.into_iter());
    result
}

#[proc_macro_attribute]
pub fn get(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut result = input.clone();
    let args = parse_macro_input!(args as route::RouteArgs<route::Get>);
    let parsed = parse_macro_input!(input as ItemFn);
    let additional: TokenStream = route::route_impl(args, parsed).into();
    result.extend(additional.into_iter());
    result
}

#[proc_macro_attribute]
pub fn post(args: TokenStream, input: TokenStream) -> TokenStream {
    let mut result = input.clone();
    let args = parse_macro_input!(args as route::RouteArgs<route::Post>);
    let parsed = parse_macro_input!(input as ItemFn);
    let additional: TokenStream = route::route_impl(args, parsed).into();
    result.extend(additional.into_iter());
    result
}
