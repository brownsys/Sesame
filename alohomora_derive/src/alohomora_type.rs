extern crate proc_macro2;
extern crate quote;
extern crate syn;

use std::iter::FromIterator;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Paren, Pound};
use syn::{Data, DeriveInput, Fields, Type, ItemStruct, Attribute, AttrStyle, Meta, MetaList, PathSegment, Path, MacroDelimiter, PathArguments, FieldsNamed};
use attribute_derive::FromAttr;

pub type Error = (Span, &'static str);

// Attributes that developers can provide to customize our derive macro.
#[derive(FromAttr)]
#[attribute(ident = alohomora_out_type)]
struct AlohomoraTypeArgs {
  name: Option<Ident>,
  to_derive: Option<Vec<Ident>>, 
}

// Generate #[derive(...)] for all the required traits for the output type.
fn derive_traits_for_output_type(attrs: &AlohomoraTypeArgs) -> Option<Attribute> {
    let trait_vec: Vec<Ident> = attrs.to_derive.clone().unwrap_or(vec![]);
    if trait_vec.len() == 0 {
        return None;
    }

    Some(Attribute {
        pound_token: Pound::default(),
        style: AttrStyle::Outer,
        bracket_token: Bracket::default(),
        meta: Meta::List(MetaList {
            path: Path {
                leading_colon: None,
                segments: Punctuated::from_iter(
                    [
                        PathSegment {
                            ident: Ident::new("derive", Span::call_site()),
                            arguments: PathArguments::None,
                        }
                    ]
                ),
            },
            delimiter: MacroDelimiter::Paren(Paren::default()),
            tokens: quote!{ #(#trait_vec),* },
        }),
    })
}

// Parse DeriveInput to a struct.
pub fn parse_derive_input_struct(input: DeriveInput) -> Result<ItemStruct, Error> {
    match input.data {
        Data::Enum(_) => Err((input.ident.span(), "derive(AlohomoraType) only works on structs")),
        Data::Union(_) => Err((input.ident.span(), "derive(AlohomoraType) only works on structs")),
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

// Construct the fields of the out type.
fn construct_out_fields(input: &ItemStruct) -> Result<Fields, Error> {
    match &input.fields {
        Fields::Named(fields) => Ok(
            Fields::Named(FieldsNamed {
                brace_token: Brace::default(),
                named: fields.named.iter()
                    .map( | field| {
                        let mut field = field.clone();
                        let ty = field.ty;
                        field.ty = Type::Verbatim(quote! {
                            <#ty as ::alohomora::AlohomoraType>::Out
                        });
                        field
                    })
                    .collect(),
            })
        ),
        _ => Err((input.ident.span(), "derive(AlohomoraType) only works on structs with named fields"))
    }
}

// Construct the entirety of the output type.
fn construct_out_type(input: &ItemStruct, attrs: &AlohomoraTypeArgs) -> Result<ItemStruct, Error> {
    let mut result = input.clone();
    result.attrs = Vec::new();
    if let Some(attr) = derive_traits_for_output_type(attrs) {
        result.attrs.push(attr);
    }
    result.ident = match &attrs.name {
        None => Ident::new(&format!("{}Out", input.ident), Span::call_site()),
        Some(name) => name.clone(),
    };
    result.fields = construct_out_fields(input)?;
    Ok(result)
}


pub fn derive_alohomora_type_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    // Parse the provided input attributes.
    let attrs = AlohomoraTypeArgs::from_attributes(&input.attrs).unwrap();

    // Parse the input struct.
    let input = parse_derive_input_struct(input)?;

    // Construct the output struct.
    let out = construct_out_type(&input, &attrs)?;

    // The generics of the input type.
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Expand needed variables.
    let input_ident = &input.ident;
    let out_ident = &out.ident;

    let fields_idents: Vec<_> = input.fields.iter().map(|field| field.ident.as_ref().unwrap()).collect();
    let fields_strings: Vec<_> = input.fields.iter().map(|field| field.ident.as_ref().unwrap().to_string()).collect();
    let fields_types: Vec<_> = input.fields.iter().map(|field| &field.ty).collect();

    Ok(quote! {
        #out

        #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
        impl #impl_generics ::alohomora::AlohomoraType for #input_ident #ty_generics #where_clause {
            type Out = #out_ident;
            fn to_enum(self) -> ::alohomora::AlohomoraTypeEnum {
                let mut map: ::std::collections::HashMap<::std::string::String, ::alohomora::AlohomoraTypeEnum> = ::std::collections::HashMap::new();
                ::alohomora::AlohomoraTypeEnum::Struct(::std::collections::HashMap::from([
                    #((String::from(#fields_strings), self.#fields_idents.to_enum()),)*
                ]))
            }
            fn from_enum(e: ::alohomora::AlohomoraTypeEnum) -> Result<Self::Out, ()> {
                match e {
                  ::alohomora::AlohomoraTypeEnum::Struct(mut hashmap) => Ok(Self::Out {
                    #(#fields_idents: <#fields_types as ::alohomora::AlohomoraType>::from_enum(hashmap.remove(#fields_strings).unwrap())?),*
                  }),
                  _ => Err(()),
                }
            }
        }
    })
}
