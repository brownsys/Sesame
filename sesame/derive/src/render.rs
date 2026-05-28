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

fn derive_struct(data: DataStruct) -> TokenStream {
    match data.fields {
        Fields::Named(fields) => {
            let puts = fields.named.into_iter().map(|field| {
                let ident = field.ident.unwrap();
                let name = ident.to_string();
                quote! {
                    map.insert(::std::string::String::from(#name), ::sesame_rocket::render::RenderFieldHelper(&self.#ident).render_field());
                }
            });
            quote! {
                let mut map: ::std::collections::BTreeMap<::std::string::String, ::sesame_rocket::render::Renderable<'__impl_pcon_render>> = ::std::collections::BTreeMap::new();
                #(#puts)*
                ::sesame_rocket::render::Renderable::Dict(map)
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
                ::sesame_rocket::render::Renderable::Dict(::std::collections::BTreeMap::new())
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
            let idents: Vec<_> = fields
                .named
                .iter()
                .map(|f| f.ident.as_ref().unwrap())
                .collect();
            let names: Vec<String> = idents.iter().map(|i| i.to_string()).collect();
            quote! {
                Self::#variant_ident { #(#idents),* } => {
                    let mut inner: ::std::collections::BTreeMap<::std::string::String, ::sesame_rocket::render::Renderable<'__impl_pcon_render>> = ::std::collections::BTreeMap::new();
                    #( inner.insert(::std::string::String::from(#names), ::sesame_rocket::render::RenderFieldHelper(#idents).render_field()); )*
                    let mut outer: ::std::collections::BTreeMap<::std::string::String, ::sesame_rocket::render::Renderable<'__impl_pcon_render>> = ::std::collections::BTreeMap::new();
                    outer.insert(::std::string::String::from(#variant_name), ::sesame_rocket::render::Renderable::Dict(inner));
                    ::sesame_rocket::render::Renderable::Dict(outer)
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
                        let mut outer: ::std::collections::BTreeMap<::std::string::String, ::sesame_rocket::render::Renderable<'__impl_pcon_render>> = ::std::collections::BTreeMap::new();
                        outer.insert(::std::string::String::from(#variant_name), ::sesame_rocket::render::RenderFieldHelper(f0).render_field());
                        ::sesame_rocket::render::Renderable::Dict(outer)
                    },
                }
            } else {
                quote! {
                    Self::#variant_ident(#(#bindings),*) => {
                        let mut outer: ::std::collections::BTreeMap<::std::string::String, ::sesame_rocket::render::Renderable<'__impl_pcon_render>> = ::std::collections::BTreeMap::new();
                        outer.insert(
                            ::std::string::String::from(#variant_name),
                            ::sesame_rocket::render::Renderable::Array(vec![#(::sesame_rocket::render::RenderFieldHelper(#bindings).render_field()),*]),
                        );
                        ::sesame_rocket::render::Renderable::Dict(outer)
                    },
                }
            }
        }
    }
}
