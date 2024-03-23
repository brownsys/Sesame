extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Ident, ItemStruct, Lit, Token};

struct NamedArg {
    ident: Ident,
    _assign: Token![=],
    value: Lit,
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

// Args are either unnamed (e.g. ("table", <index>))
// or named (e.g. (table="table", column=1)).
type UnnamedArgs = syn::punctuated::Punctuated<Lit, Token![,]>;
type NamedArgs = syn::punctuated::Punctuated<NamedArg, Token![,]>;

// Parsed arguments.
pub struct SchemaPolicyArgs {
    table: String,
    column: usize,
}
impl Parse for SchemaPolicyArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let named = NamedArgs::parse_terminated(input);
        match named {
            Result::Ok(named) => {
                if named.len() != 2 {
                    panic!("#schema_policy called with wrong number of arguments");
                }

                let mut table: Option<String> = None;
                let mut column: Option<usize> = None;
                for arg in named.iter() {
                    let prop = arg.ident.to_string();
                    if prop == "table" {
                        if let Lit::Str(value) = &arg.value {
                            table = Option::Some(value.value());
                        } else {
                            panic!("table assigned non-string value");
                        };
                    } else if prop == "column" {
                        if let Lit::Int(value) = &arg.value {
                            column = Option::Some(value.base10_parse()?);
                        } else {
                            panic!("column assigned non-int value");
                        };
                    }
                }
                Ok(SchemaPolicyArgs {
                    table: table.unwrap(),
                    column: column.unwrap(),
                })
            }
            Result::Err(_) => {
                let unnamed = UnnamedArgs::parse_terminated(input)?;
                if unnamed.len() != 2 {
                    panic!("#schema_policy called with wrong number of arguments");
                }

                let table = if let Lit::Str(value) = &unnamed[0] {
                    value.value()
                } else {
                    panic!("table assigned non-string value (unnamed)");
                };
                let column = if let Lit::Int(value) = &unnamed[1] {
                    value.base10_parse()?
                } else {
                    panic!("column assigned non-int value (unnamed)");
                };
                Ok(SchemaPolicyArgs {
                    table: table,
                    column: column,
                })
            }
        }
    }
}

pub fn schema_policy_impl(args: SchemaPolicyArgs, input: ItemStruct) -> TokenStream {
    let name = input.ident;
    let table = args.table;
    let column = args.column;
    let fname = format!("register_{}_{}_{}", name, table, column);
    let func_name = Ident::new(&fname, proc_macro2::Span::call_site());

    quote! {
      #[::alohomora::policy::register]
      unsafe fn #func_name() {
          ::alohomora::policy::add_schema_policy::<#name>(::std::string::String::from(#table), #column);
      }
    }
}
