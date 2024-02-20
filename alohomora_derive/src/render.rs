extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Data, DataStruct, DeriveInput, Field, Fields};

pub fn derive_boxed_serialize_impl(input: DeriveInput) -> TokenStream {
    // struct name we are deriving for.
    let input_name = input.ident;

    // fields inside struct.
    let fields: Punctuated<Field, Comma> = match input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => fields.named,
        _ => panic!("this derive macro only works on structs with named fields"),
    };

    let puts = fields.into_iter().map(|field| {
        let field = field.ident.unwrap();
        let field_name: String = field.to_string();

        quote! {
          map.insert(::std::string::String::from(#field_name), self.#field.render());
        }
    });

    // Generics if any.
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // Impl trait.
    quote! {
      #[automatically_derived]
      impl #impl_generics ::alohomora::bbox::BBoxRender for #input_name #ty_generics #where_clause {
        fn render<'a>(&'a self) -> ::alohomora::bbox::Renderable<'a> {
          let mut map: ::std::collections::BTreeMap<::std::string::String, ::alohomora::bbox::Renderable<'a>> = ::std::collections::BTreeMap::new();
          #(#puts)*
          ::alohomora::bbox::Renderable::Dict(map)
        }
      }
    }
}
