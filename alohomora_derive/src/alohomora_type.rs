extern crate proc_macro2;
extern crate quote;
extern crate syn;

use std::iter::FromIterator;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::{Brace, Bracket, Paren, Pound};
use syn::{Data, DeriveInput, Fields, Type, ItemStruct, Attribute, AttrStyle, Meta, MetaList, PathSegment, Path, MacroDelimiter, PathArguments, FieldsNamed, Generics, Token, GenericParam, TypeParam, TypeParamBound, TraitBound, TraitBoundModifier, WhereClause, WherePredicate, PredicateType, AngleBracketedGenericArguments, GenericArgument, TypePath};
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

fn make_tdyn() -> GenericParam {
    GenericParam::Type(TypeParam {
        attrs: Vec::new(),
        ident: Ident::new("__TDyn", Span::mixed_site()),
        colon_token: Default::default(),
        bounds: Punctuated::from_iter([
            TypeParamBound::Trait(
                TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: Some(Default::default()),
                        segments: Punctuated::from_iter([
                            PathSegment {
                                ident: Ident::new("alohomora", Span::mixed_site()),
                                arguments: PathArguments::None,
                            },
                            PathSegment {
                                ident: Ident::new("sesame_type_dyns", Span::mixed_site()),
                                arguments: PathArguments::None,
                            },
                            PathSegment {
                                ident: Ident::new("SesameDyn", Span::mixed_site()),
                                arguments: PathArguments::None,
                            }
                        ])
                    }
                }
            ),
            TypeParamBound::Trait(
                TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::Maybe(Default::default()),
                    lifetimes: None,
                    path: Path {
                        leading_colon: None,
                        segments: Punctuated::from_iter([
                            PathSegment {
                                ident: Ident::new("Sized", Span::mixed_site()),
                                arguments: PathArguments::None,
                            }
                        ])
                    }
                }
            )
        ]),
        eq_token: None,
        default: None,
    })
}
fn make_pdyn() -> GenericParam {
    GenericParam::Type(TypeParam {
        attrs: Vec::new(),
        ident: Ident::new("__PDyn", Span::mixed_site()),
        colon_token: Default::default(),
        bounds: Punctuated::from_iter([
            TypeParamBound::Trait(
                TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: Some(Default::default()),
                        segments: Punctuated::from_iter([
                            PathSegment {
                                ident: Ident::new("alohomora", Span::mixed_site()),
                                arguments: PathArguments::None,
                            },
                            PathSegment {
                                ident: Ident::new("policy", Span::mixed_site()),
                                arguments: PathArguments::None,
                            },
                            PathSegment {
                                ident: Ident::new("PolicyDyn", Span::mixed_site()),
                                arguments: PathArguments::None,
                            }
                        ])
                    }
                }
            ),
            TypeParamBound::Trait(
                TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::Maybe(Default::default()),
                    lifetimes: None,
                    path: Path {
                        leading_colon: None,
                        segments: Punctuated::from_iter([
                            PathSegment {
                                ident: Ident::new("Sized", Span::mixed_site()),
                                arguments: PathArguments::None,
                            }
                        ])
                    }
                }
            )
        ]),
        eq_token: None,
        default: None,
    })
}

// Create a where clause bound on the form
// $field_type: ::alohomora::SesameType<__TDyn, __PDyn>
fn sesame_type_where_clause(field_type: &Type) -> WherePredicate {
    WherePredicate::Type(PredicateType {
        lifetimes: None,
        bounded_ty: field_type.clone(),
        colon_token: Default::default(),
        bounds: Punctuated::from_iter([
            TypeParamBound::Trait(
                TraitBound {
                    paren_token: None,
                    modifier: TraitBoundModifier::None,
                    lifetimes: None,
                    path: Path {
                        leading_colon: Some(Default::default()),
                        segments: Punctuated::from_iter([
                            PathSegment {
                                ident: Ident::new("alohomora", Span::mixed_site()),
                                arguments: PathArguments::None,
                            },
                            PathSegment {
                                ident: Ident::new("SesameType", Span::mixed_site()),
                                arguments: PathArguments::AngleBracketed(
                                    AngleBracketedGenericArguments {
                                        colon2_token: Default::default(),
                                        lt_token: Default::default(),
                                        args: Punctuated::from_iter([
                                            GenericArgument::Type(
                                                Type::Path(
                                                    TypePath {
                                                        qself: None,
                                                        path: Path {
                                                            leading_colon: None,
                                                            segments: Punctuated::from_iter([
                                                                PathSegment {
                                                                    ident: Ident::new("__TDyn", Span::mixed_site()),
                                                                    arguments: PathArguments::None,
                                                                }
                                                            ])
                                                        }
                                                    }
                                                )
                                            ),
                                            GenericArgument::Type(
                                                Type::Path(
                                                    TypePath {
                                                        qself: None,
                                                        path: Path {
                                                            leading_colon: None,
                                                            segments: Punctuated::from_iter([
                                                                PathSegment {
                                                                    ident: Ident::new("__PDyn", Span::mixed_site()),
                                                                    arguments: PathArguments::None,
                                                                }
                                                            ])
                                                        }
                                                    }
                                                )
                                            )
                                        ]),
                                        gt_token: Default::default(),
                                    }
                                ),
                            },
                        ])
                    }
                }
            )]
        )
    })
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
        Data::Enum(_) => Err((input.ident.span(), "derive(SesameType) only works on structs")),
        Data::Union(_) => Err((input.ident.span(), "derive(SesameType) only works on structs")),
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
                                <#ty as ::alohomora::SesameTypeOut>::Out
                            });
                        }
                        field
                    })
                    .collect(),
            })
        ),
        _ => Err((input.ident.span(), "derive(SesameType) only works on structs with named fields"))
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
    result.fields = construct_out_fields(&input, attrs)?;
    Ok(result)
}


pub fn derive_sesame_type_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    // Parse the provided input attributes.
    let attrs = AlohomoraTypeArgs::from_attributes(&input.attrs).unwrap();

    // Parse the input struct.
    let mut input = parse_derive_input_struct(input)?;

    // Construct the output struct.
    let out = construct_out_type(&input, &attrs)?;

    // Expand needed variables.
    let input_ident = &input.ident;
    let out_ident = &out.ident;

    // Find all fields.
    let fields: Vec<_> = input.fields.iter()
        .map(|field| (
            field.ident.as_ref().unwrap().clone(),
            field.ident.as_ref().unwrap().to_string(),
            field.ty.clone(),
        ))
        .collect();

    // Filter into those that are AlohomoraTypes themselves, and those who are kept verbatim.
    let alohomora_fields: Vec<_> = fields
        .iter()
        .filter(|(_, string, _)| !attrs.is_verbatim(string))
        .cloned()
        .collect();
    let verbatium_fields: Vec<_> = fields
        .iter()
        .filter(|(_, string, _)| attrs.is_verbatim(string))
        .cloned()
        .collect();

    // Split field components.
    let alohomora_fields_idents: Vec<_> = alohomora_fields
        .iter()
        .map(|(ident, _, _)| ident.clone())
        .collect();
    let alohomora_fields_strings: Vec<_> = alohomora_fields
        .iter()
        .map(|(_, string, _)| string.clone())
        .collect();
    let alohomora_fields_types: Vec<_> = alohomora_fields
        .iter()
        .map(|(_, _, ty)| ty.clone())
        .collect();

    let verbatim_fields_idents: Vec<_> = verbatium_fields
        .iter()
        .map(|(ident, _, _)| ident.clone())
        .collect();
    let verbatim_fields_strings: Vec<_> = verbatium_fields
        .iter()
        .map(|(_, string, _)| string.clone())
        .collect();

    // Store type traits prior to adding our own __TDyn and __PDyn
    let (impl_generics_out, ty_generics, where_clause_out) = input.generics.split_for_impl();

    // Add trait bounds for SesameDyn and PolicyDyn.
    let mut input = input.clone();
    input.generics.params.push(make_tdyn());
    input.generics.params.push(make_pdyn());

    // Add where clause bounds for each inner type.
    if input.generics.where_clause.is_none() {
        input.generics.where_clause = Some(WhereClause {
            where_token: Default::default(),
            predicates: Punctuated::new()
        })
    }
    let where_clause = input.generics.where_clause.as_mut().unwrap();
    for field_type in &alohomora_fields_types {
        where_clause.predicates.push(sesame_type_where_clause(field_type));
    }

    // The generics of the input type.
    let (impl_generics, _, where_clause) = input.generics.split_for_impl();

    // Generate implementation.
    Ok(quote! {
        #[automatically_derived]
        #out

        #[automatically_derived]
        #[doc = "Library implementation of SesameTypeOut. Do not copy this docstring!"]
        impl #impl_generics_out ::alohomora::SesameTypeOut  for #input_ident #ty_generics #where_clause_out {
            type Out = #out_ident;
        }

        #[automatically_derived]
        #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
        impl #impl_generics ::alohomora::SesameType<__TDyn, __PDyn>  for #input_ident #ty_generics #where_clause {
            fn to_enum(self) -> ::alohomora::SesameTypeEnum<__TDyn, __PDyn>  {
                let mut map: ::std::collections::HashMap<::std::string::String, ::alohomora::SesameTypeEnum> = ::std::collections::HashMap::new();
                ::alohomora::SesameTypeEnum::Struct(::std::collections::HashMap::from([
                    #((String::from(#alohomora_fields_strings), <#alohomora_fields_types as ::alohomora::SesameType<__TDyn, __PDyn> >::to_enum(self.#alohomora_fields_idents)),)*
                    #((String::from(#verbatim_fields_strings), ::alohomora::SesameTypeEnum::Value(Box::new(self.#verbatim_fields_idents))),)*
                ]))
            }
            fn from_enum(e: ::alohomora::SesameTypeEnum<__TDyn, __PDyn> ) -> Result<Self::Out, ()> {
                match e {
                    ::alohomora::SesameTypeEnum::Struct(mut hashmap) => {
                        Ok(Self::Out {
                            #(#alohomora_fields_idents: <#alohomora_fields_types as ::alohomora::SesameType<__TDyn, __PDyn> >::from_enum(hashmap.remove(#alohomora_fields_strings).unwrap())?,)*
                            #(#verbatim_fields_idents: hashmap.remove(#verbatim_fields_strings).unwrap().coerce()?,)*
                        })
                    },
                    _ => Err(()),
                }
            }
        }
    })
}
