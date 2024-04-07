extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Span, Ident, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, ItemStruct};
use attribute_derive::FromAttr;

pub type Error = (Span, &'static str);

// Attributes that developers can provide to customize our derive macro.
#[derive(FromAttr)]
#[attribute(ident = response_bbox_json)]
struct ResponseBBoxJsonArgs {
    as_is: Option<Vec<Ident>>,
}

impl ResponseBBoxJsonArgs {
    pub fn contains(&self, ident: &Ident) -> bool {
        match &self.as_is {
            None => false,
            Some(v) => v.contains(ident),
        }
    }
}

// Parse DeriveInput to a struct.
fn parse_derive_input_struct(input: DeriveInput) -> Result<ItemStruct, Error> {
    match input.data {
        Data::Enum(_) => Err((input.ident.span(), "derive(FromBBoxJson) only works on structs")),
        Data::Union(_) => Err((input.ident.span(), "derive(FromBBoxJson) only works on structs")),
        Data::Struct(data_struct) => Ok(
            ItemStruct {
                attrs: input.attrs,
                vis: input.vis,
                struct_token: data_struct.struct_token,
                ident: input.ident,
                generics: input.generics,
                fields: data_struct.fields,
                semi_token: data_struct.semi_token,
            }
        ),
    }
}

pub fn request_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    // Parse the input struct.
    let input = parse_derive_input_struct(input)?;

    // The generics of the input type.
    let input_ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // List all fields.
    let fields: Vec<_> = input.fields.iter().map(|field| field.ident.as_ref().unwrap()).collect();
    let fields_strings: Vec<_> = fields.iter().map(|field| field.to_string()).collect();

    // Generate implementation.
    Ok(quote! {
        impl #impl_generics ::alohomora::rocket::RequestBBoxJson for #input_ident #ty_generics #where_clause {
            fn from_json(
                mut __value: ::alohomora::rocket::InputBBoxValue,
                __request: ::alohomora::rocket::BBoxRequest<'_, '_>,
            ) -> Result<Self, &'static str> {
                Ok(Self {
                    #(#fields: __value.get(#fields_strings)?.into_json(__request)?),*
                })
            }
        }
    })
}

pub fn response_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    // Parse the provided input attributes.
    let attrs = ResponseBBoxJsonArgs::from_attributes(&input.attrs).unwrap();

    // Parse the input struct.
    let input = parse_derive_input_struct(input)?;

    // The generics of the input type.
    let input_ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // List all fields.
    let all_fields: Vec<_> = input.fields.iter().map(|field| field.ident.as_ref().unwrap()).collect();

    let fields: Vec<_> = all_fields.iter().filter(|field| !attrs.contains(field)).collect();
    let fields_strings: Vec<_> = fields.iter().map(|field| field.to_string()).collect();

    let as_is: Vec<_> = all_fields.iter().filter(|field| attrs.contains(field)).collect();
    let as_is_strings: Vec<_> = as_is.iter().map(|field| field.to_string()).collect();

    // Generate implementation.
    Ok(quote! {
        impl #impl_generics ::alohomora::rocket::ResponseBBoxJson for #input_ident #ty_generics #where_clause {
            fn to_json(self) -> ::alohomora::rocket::OutputBBoxValue {
                ::alohomora::rocket::OutputBBoxValue::Object(HashMap::from([
                    #((String::from(#fields_strings), self.#fields.to_json()),)*
                    #((
                        String::from(#as_is_strings),
                        ::alohomora::rocket::OutputBBoxValue::Value(serde_json::to_value(self.#as_is).unwrap()),
                    )),*
                ]))
            }
        }
    })
}
