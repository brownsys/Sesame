extern crate proc_macro;
extern crate syn;

use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{parse_macro_input, DeriveInput, Expr, ItemFn, ItemStruct};

mod form;
mod policy;
mod render;
mod route;
mod alohomora_type;
mod sandbox;
mod json;

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

#[proc_macro]
pub fn routes(input: TokenStream) -> TokenStream {
    use syn::parse::Parser;
    let parser = Punctuated::<Expr, Comma>::parse_terminated;
    let input = parser.parse(input).unwrap();
    let input = input.iter();
    let result = quote! {
        vec![#( #input ::info().into()),*]
    };
    result.into()
}

#[proc_macro_derive(AlohomoraType, attributes(alohomora_out_type))]
pub fn derive_alohomora_type(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match alohomora_type::derive_alohomora_type_impl(input) {
        Ok(tokens) => tokens.into(),
        Err((span, err)) => quote_spanned!(span => compile_error!(#err)).into(),
    }
}

#[allow(non_snake_case)]
#[proc_macro_attribute]
pub fn AlohomoraSandbox(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut result = input.clone();
    let parsed = parse_macro_input!(input as ItemFn);
    let additional: TokenStream = sandbox::sandbox_impl(parsed).into();
    result.extend(additional.into_iter());
    result
}

#[proc_macro_derive(FastTransfer)]
pub fn derive_sandboxable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match sandbox::derive_fast_transfer_impl(input) {
        Ok(tokens) => tokens.into(),
        Err((span, err)) => quote_spanned!(span => compile_error!(#err)).into(),
    }
}


#[proc_macro_derive(RequestBBoxJson)]
pub fn derive_request_bbox_json(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match json::request_impl(input) {
        Ok(tokens) => tokens.into(),
        Err((span, err)) => quote_spanned!(span => compile_error!(#err)).into(),
    }
}

#[proc_macro_derive(ResponseBBoxJson, attributes(response_bbox_json))]
pub fn dervie_response_bbox_json(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match json::response_impl(input) {
        Ok(tokens) => tokens.into(),
        Err((span, err)) => quote_spanned!(span => compile_error!(#err)).into(),
    }
}
