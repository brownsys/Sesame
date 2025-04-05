use std::fmt::Debug;

use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse_macro_input, Attribute, Ident, Item, ItemStruct, Lit, Token};

mod internal_unchecked;
mod foreign;
mod company;

pub enum Domain {
    Internal,
    Company,
    Foreign,
}

struct NamedArg {
    ident: Ident,
    _assign: Token![=],
    value: Ident,
}

impl Debug for NamedArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Attribute is {}={}", self.ident.to_string(), self.value.to_string())
    }

}

impl Parse for NamedArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ident: input.parse()?,
            _assign: input.parse()?,
            value: input.parse()?,
        })
    }
}

type UnnamedArgs = syn::punctuated::Punctuated<Lit, Token![,]>;
type NamedArgs = syn::punctuated::Punctuated<NamedArg, Token![,]>;

impl Parse for Domain {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let named: syn::Result<NamedArg> = input.parse();
        // panic!("{:?}", dodo);
        // let named = NamedArgs::parse_terminated(input);
        match named {
            Result::Ok(domain) => {
                // if named.len() != 1 {
                //     panic!(
                //         "Wrong number of arguments to tahini service generation. Got{}",
                //         named.len()
                //     )
                // }
                // let domain = &named[0];
                let prop = domain.ident.to_string();
                if prop != "domain" {
                    panic!("Only argument passed was not domain")
                }
                let val = domain.value.to_string();
                match val.to_lowercase().as_str() {
                    "internal" => Ok(Domain::Internal),
                    "company" => Ok(Domain::Company),
                    "foreign" => Ok(Domain::Foreign),
                    _ => panic!("Domain assigned unknown value"),
                }
            }
            Result::Err(_) => panic!("Couldn't parse arguments"),
        }
    }
}

pub fn service(attrs: TokenStream, input: TokenStream) -> TokenStream {
    let domain = parse_macro_input!(attrs as Domain);
    let attrs = TokenStream::new();
    match domain {
        Domain::Internal => internal_unchecked::service(attrs, input),
        Domain::Company => company::service(attrs, input),
        Domain::Foreign => foreign::service(attrs, input)
    }
}
