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
  verbatim: Option<Vec<Ident>>,
}
impl AlohomoraTypeArgs {
    pub fn is_verbatim(&self, ident: &str) -> bool {
        match &self.verbatim {
            None => false,
            Some(v) => {
                for i in v {
                    if &i.to_string() == ident {
                        return true;
                    }
                }
                false
            },
        }
    }
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
fn construct_out_fields(input: &ItemStruct, attrs: &AlohomoraTypeArgs) -> Result<Fields, Error> {
    match &input.fields {
        Fields::Named(fields) => Ok(
            Fields::Named(FieldsNamed {
                brace_token: Brace::default(),
                named: fields.named.iter()
                    .map( | field| {
                        let mut field = field.clone();
                        let ty = &field.ty;
                        if !attrs.is_verbatim(&field.ident.as_ref().unwrap().to_string()) {
                            field.ty = Type::Verbatim(quote! {
                                <#ty as ::alohomora::AlohomoraType>::Out
                            });
                        }
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
        None => Ident::new(&format!("{}Out", input.ident), Span::mixed_site()),
        Some(name) => name.clone(),
    };
    result.fields = construct_out_fields(input, attrs)?;
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

    // Find all fields.
    let fields: Vec<_> = input.fields.iter()
        .map(|field| field.ident.as_ref().unwrap().clone())
        .collect();

    // Filter into those that are AlohomoraTypes themselves, and those who are kept verbatim.
    let alohomora_fields: Vec<_> = fields
        .iter()
        .filter(|ident| !attrs.is_verbatim(&ident.to_string()))
        .cloned()
        .collect();
    let verbatium_fields: Vec<_> = fields
        .iter()
        .filter(|ident| attrs.is_verbatim(&ident.to_string()))
        .cloned()
        .collect();

    // Split field components.
    let alohomora_fields_t: Vec<_> = alohomora_fields
        .iter()
        .map(|ident| Ident::new(&format!("__{}_t", ident), ident.span()))
        .collect();
    let alohomora_fields_p: Vec<_> = alohomora_fields
        .iter()
        .map(|ident| Ident::new(&format!("__{}_p", ident), ident.span()))
        .collect();

    // Pairs of joins.
    let mut joins_left = Vec::new();
    let mut joins_right = Vec::new();
    for i in 1..alohomora_fields_p.len() {
        joins_left.push(alohomora_fields_p[i-1].clone());
        joins_right.push(alohomora_fields_p[i].clone());
    }
    let last_policy = if alohomora_fields_p.len() > 0 {
        let ident = alohomora_fields_p[alohomora_fields_p.len() - 1].clone();
        quote! {
            #ident
        }
    } else {
        quote! {
            ::alohomora::policy::AnyPolicy::new(::alohomora::policy::NoPolicy {})
        }
    };

    // Generate implementation.
    Ok(quote! {
        #[automatically_derived]
        #out

        #[automatically_derived]
        #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
        impl #impl_generics ::alohomora::AlohomoraType for #input_ident #ty_generics #where_clause {
            type Out = #out_ident;
            type Policy = ::alohomora::policy::AnyPolicy;
            fn inner_fold(self, unwrapper: &::alohomora::Unwrapper) -> Result<(Self::Out, Self::Policy), ()> {
                use ::alohomora::AlohomoraType;
                use ::alohomora::policy::Policy;
                #(let (#alohomora_fields_t, #alohomora_fields_p) = self.#alohomora_fields.inner_fold(unwrapper)?;)*
                #(let #joins_right = #joins_left.join(::alohomora::policy::AnyPolicy::new(#joins_right))?;)*
                Ok((
                    Self::Out {
                        #(#alohomora_fields: #alohomora_fields_t,)*
                        #(#verbatium_fields: self.#verbatium_fields,)*
                    },
                    #last_policy,
                ))
            }
        }
    })
}
