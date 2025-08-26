extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput};

pub type Error = (Span, &'static str);

// Parse DeriveInput to a struct.
pub fn parse_input_struct_name(input: DeriveInput) -> Result<Ident, Error> {
    match input.data {
        Data::Enum(_) => Err((input.ident.span(), "derive(NoFoldIn) only works on structs")),
        Data::Union(_) => Err((input.ident.span(), "derive(NoFoldIn) only works on structs")),
        Data::Struct(_) => Ok(input.ident),
    }
}

pub fn derive_no_fold_in_impl(input: DeriveInput) -> Result<TokenStream, Error> {
    // Parse the input struct.
    let input_ident: Ident = parse_input_struct_name(input.clone())?;

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    // Generate implementation.
    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics !::alohomora::fold_in::FoldInAllowed for #input_ident #ty_generics #where_clause {}
    })
}
