extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Data, DataEnum, DataStruct, DeriveInput, Fields, Variant};

pub fn derive_boxed_serialize_impl(input: DeriveInput) -> TokenStream {
    let input_name = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let body = match input.data {
        Data::Struct(data) => derive_struct(data),
        Data::Enum(data) => derive_enum(data),
        _ => panic!("PConRender can only be derived for structs and enums"),
    };

    quote! {
        #[automatically_derived]
        impl #impl_generics ::sesame_rocket::render::PConRender for #input_name #ty_generics #where_clause {
            fn render<'__impl_pcon_render>(&'__impl_pcon_render self) -> ::sesame_rocket::render::Renderable<'__impl_pcon_render> {
                use ::sesame_rocket::render::SerializeFieldFallback as _;
                #body
            }
        }
    }
}

// Pairs every named field with its key, ordered by key rather than by
// declaration. Names are known at expansion time, so the ordering costs nothing
// at runtime, and it means `transform` inserts into its output
// `serde_json::Map` -- a `BTreeMap` -- in ascending order, which is that map's
// cheapest insertion pattern. The `BTreeMap`-based `Renderable::Dict` used to
// hand it sorted keys for free; a flat `Vec` in declaration order would not.
fn sorted_fields<'a>(
    fields: impl Iterator<Item = &'a syn::Field>,
) -> Vec<(String, &'a syn::Ident)> {
    let mut named: Vec<(String, &syn::Ident)> = fields
        .map(|field| {
            let ident = field.ident.as_ref().unwrap();
            (ident.to_string(), ident)
        })
        .collect();
    named.sort_by(|(a, _), (b, _)| a.cmp(b));
    named
}

fn derive_struct(data: DataStruct) -> TokenStream {
    match data.fields {
        Fields::Named(fields) => {
            let puts = sorted_fields(fields.named.iter()).into_iter().map(|(name, ident)| {
                quote! {
                    (#name, ::sesame_rocket::render::RenderFieldHelper(&self.#ident).render_field()),
                }
            });
            quote! {
                ::sesame_rocket::render::Renderable::Object(::std::vec::Vec::from([
                    #(#puts)*
                ]))
            }
        }
        Fields::Unnamed(fields) => {
            let puts = fields.unnamed.into_iter().enumerate().map(|(i, _)| {
                let index = syn::Index::from(i);
                quote! { ::sesame_rocket::render::RenderFieldHelper(&self.#index).render_field() }
            });
            quote! {
                ::sesame_rocket::render::Renderable::Array(vec![#(#puts),*])
            }
        }
        Fields::Unit => {
            quote! {
                ::sesame_rocket::render::Renderable::Object(::std::vec::Vec::new())
            }
        }
    }
}

fn derive_enum(data: DataEnum) -> TokenStream {
    let arms = data.variants.into_iter().map(derive_variant);
    quote! {
        match self {
            #(#arms)*
        }
    }
}

fn derive_variant(variant: Variant) -> TokenStream {
    let variant_ident = &variant.ident;
    let variant_name = variant_ident.to_string();

    match variant.fields {
        Fields::Unit => {
            quote! {
                Self::#variant_ident => ::sesame_rocket::render::Renderable::Serialize(&#variant_name),
            }
        }
        Fields::Named(fields) => {
            let sorted = sorted_fields(fields.named.iter());
            let names: Vec<&String> = sorted.iter().map(|(name, _)| name).collect();
            let idents: Vec<&syn::Ident> = sorted.iter().map(|(_, ident)| *ident).collect();
            quote! {
                Self::#variant_ident { #(#idents),* } => {
                    ::sesame_rocket::render::Renderable::Object(::std::vec::Vec::from([(
                        #variant_name,
                        ::sesame_rocket::render::Renderable::Object(::std::vec::Vec::from([
                            #( (#names, ::sesame_rocket::render::RenderFieldHelper(#idents).render_field()), )*
                        ])),
                    )]))
                },
            }
        }
        Fields::Unnamed(fields) => {
            let count = fields.unnamed.len();
            let bindings: Vec<_> = (0..count)
                .map(|i| {
                    syn::Ident::new(&format!("f{}", i), proc_macro2::Span::call_site())
                })
                .collect();

            if count == 1 {
                quote! {
                    Self::#variant_ident(#(#bindings),*) => {
                        ::sesame_rocket::render::Renderable::Object(::std::vec::Vec::from([(
                            #variant_name,
                            ::sesame_rocket::render::RenderFieldHelper(f0).render_field(),
                        )]))
                    },
                }
            } else {
                quote! {
                    Self::#variant_ident(#(#bindings),*) => {
                        ::sesame_rocket::render::Renderable::Object(::std::vec::Vec::from([(
                            #variant_name,
                            ::sesame_rocket::render::Renderable::Array(vec![#(::sesame_rocket::render::RenderFieldHelper(#bindings).render_field()),*]),
                        )]))
                    },
                }
            }
        }
    }
}
