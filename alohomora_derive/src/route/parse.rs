extern crate proc_macro2;
extern crate syn;

use std::marker::PhantomData;
use std::option::Option;
use std::result::Result;

use proc_macro2::{Ident, Span};
use syn::parse::{Parse, ParseStream};
use syn::{parse_str, Lit, Token};

use rocket_http::uri::Origin;

// A parameter of the form <IDENTIFIER>.
fn is_dynamic_parameter(s: &str) -> bool {
    s.starts_with('<') && s.ends_with('>')
}
fn parse_parameter(s: &str) -> Option<String> {
    // Ensure literal is on the form `<...>`.
    if !is_dynamic_parameter(s) {
        return Option::None;
    }

    // Ensure literal is an identifier.
    let s: &str = &s[1..(s.len() - 1)];
    if let Err(_) = parse_str::<Ident>(s) {
        return Option::None;
    }

    Option::Some(String::from(s))
}

// Method.
enum Method {
    Post,
    Get,
    Delete,
}
impl Method {
    pub fn from_string(method: &str) -> Option<Self> {
        match method {
            "POST" => Some(Method::Post),
            "GET" => Some(Method::Get),
            "DELETE" => Some(Method::Delete),
            _ => None,
        }
    }
    pub fn to_ident(&self) -> Ident {
        match self {
            Method::Post => Ident::new("Post", Span::call_site()),
            Method::Get => Ident::new("Get", Span::call_site()),
            Method::Delete => Ident::new("Delete", Span::call_site()),
        }
    }
}
impl Parse for Method {
    fn parse(stream: ParseStream) -> Result<Self, syn::Error> {
        let ident: Ident = stream.fork().parse()?;
        let ident = ident.to_string();
        match Method::from_string(&ident) {
            None => Err(stream.error(format!("Bad method {}", ident))),
            Some(method) => {
                stream.parse::<Ident>()?;
                Ok(method)
            }
        }
    }
}

// RouteURI with dynamic parameters (e.g. /list/<num>/something?<var>&<x>)
struct RouteURI {
    pub uri: String,
    pub query_params: Vec<String>,
    pub path_params: Vec<(String, usize)>,
}
impl Parse for RouteURI {
    fn parse(stream: ParseStream) -> Result<Self, syn::Error> {
        let lit: Lit = stream.fork().parse()?;

        // Extract URI literal.
        let uri = match lit {
            Lit::Str(uri) => uri.value(),
            _ => {
                return Err(stream.error("Expected a literal str for URI"));
            }
        };

        // Parse route.
        let origin = match Origin::parse_route(&uri) {
            Ok(origin) => origin,
            Err(e) => {
                return Err(stream.error(e));
            }
        };

        // Make sure route is normalized.
        if !origin.is_normalized() {
            return Err(stream.error("Expected route URI to be normalized"));
        }

        // Extract path parameters (e.g. ../<x>/..)
        let mut path_params = Vec::new();
        let path = origin.path();
        let params = path
            .split('/')
            .filter(|s| s.len() > 0)
            .enumerate()
            .filter(|(_i, s)| is_dynamic_parameter(s.as_ref()));
        for (i, param) in params {
            let param = match parse_parameter(param.as_ref()) {
                Option::Some(param) => param,
                Option::None => {
                    return Err(stream.error("dynamic path parameter must be a valid identifier"));
                }
            };
            path_params.push((param, i));
        }

        // Extract query parameters (e.g. ../..?<x>)
        let query_it = match origin.query() {
            None => Vec::new(),
            Some(query) => query
                .split('&')
                .filter(|s| !s.is_empty())
                .map(|s| String::from(s.as_str()))
                .collect(),
        };

        let mut query_params = Vec::new();
        for param in query_it {
            if !is_dynamic_parameter(&param) {
                return Err(stream.error("Only dynamic query parameters are supported"));
            }
            let param = match parse_parameter(&param) {
                Option::Some(param) => param,
                Option::None => {
                    return Err(
                        stream.error("dynamic query parameter name must be a valid identifier")
                    );
                }
            };
            query_params.push(param);
        }

        // All good.
        stream.parse::<Lit>()?;
        Ok(RouteURI {
            uri,
            query_params,
            path_params,
        })
    }
}

// data = "<data>".
struct DataArg {
    pub parameter: String,
}
impl Parse for DataArg {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        // Parse data ident.
        let fork = input.fork();
        let ident: Ident = fork.parse()?;
        if ident.to_string() != "data" {
            return Err(input.error("Expected data ="));
        }
        input.parse::<Ident>()?;

        // Parse = token.
        let _token: Token![=] = input.parse()?;

        // Parse str literal.
        let fork = input.fork();
        let lit: Lit = fork.parse()?;
        let data = match lit {
            Lit::Str(lit) => lit.value(),
            _ => {
                return Err(input.error("Expected a literal str for data"));
            }
        };

        // Ensure literal is on the form `<...>`.
        let parameter = match parse_parameter(&data) {
            Option::Some(data) => data,
            Option::None => {
                return Err(input.error("Expected <IDENTIFIER> for data"));
            }
        };

        // Done.
        input.parse::<Lit>()?;
        Ok(DataArg { parameter })
    }
}


// with_data = "<data>".
struct WithDataArg {
    pub parameter: String,
}
impl Parse for WithDataArg {
    fn parse(input: ParseStream) -> Result<Self, syn::Error> {
        // Parse data ident.
        let fork = input.fork();
        let ident: Ident = fork.parse()?;
        if ident.to_string() != "with_data" {
            return Err(input.error("Expected with_data ="));
        }
        input.parse::<Ident>()?;

        // Parse = token.
        let _token: Token![=] = input.parse()?;

        // Parse str literal.
        let fork = input.fork();
        let lit: Lit = fork.parse()?;
        let data = match lit {
            Lit::Str(lit) => lit.value(),
            _ => {
                return Err(input.error("Expected a literal str for data"));
            }
        };

        // Ensure literal is on the form `<...>`.
        let parameter = match parse_parameter(&data) {
            Option::Some(data) => data,
            Option::None => {
                return Err(input.error("Expected <IDENTIFIER> for data"));
            }
        };

        // Done.
        input.parse::<Lit>()?;
        Ok(WithDataArg { parameter })
    }
}

// Used to tell parser whether we know the method ahead of time or not.
pub trait RouteType {
    const TYPE: &'static str;
}

pub struct Unknown {}
impl RouteType for Unknown {
    const TYPE: &'static str = "U";
}
pub struct Get {}
impl RouteType for Get {
    const TYPE: &'static str = "GET";
}
pub struct Post {}
impl RouteType for Post {
    const TYPE: &'static str = "POST";
}

// Parsed arguments.
pub struct RouteArgs<T: RouteType> {
    pub method: Ident,
    pub uri: String,
    pub query_params: Vec<String>,
    pub path_params: Vec<(String, usize)>,
    pub data: Option<String>,
    pub with_data: Option<String>,
    _t: PhantomData<T>,
}
impl<T: RouteType> Parse for RouteArgs<T> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Use method if known, otherwise parse it!
        let method = match T::TYPE {
            Unknown::TYPE => {
                let method: Method = input.parse()?;
                let _comma: Token![,] = input.parse()?;
                method
            }
            _ => Method::from_string(T::TYPE).unwrap(),
        };

        // Parse URI.
        let uri: RouteURI = input.parse()?;

        // Parse data.
        let data: Option<String> = if !input.is_empty() {
            let _comma: Token![,] = input.parse()?;
            let data: DataArg = input.parse()?;
            Option::Some(data.parameter)
        } else {
            Option::None
        };

        // Parse with_data.
        let with_data: Option<String> = if !input.is_empty() {
            let _comma: Token![,] = input.parse()?;
            let data: WithDataArg = input.parse()?;
            Option::Some(data.parameter)
        } else {
            Option::None
        };

        // Should be empty here.
        if !input.is_empty() {
            return Err(input.error("Expected stream to be empty"));
        }

        // Return the parsed args.
        Ok(RouteArgs {
            method: method.to_ident(),
            uri: uri.uri,
            query_params: uri.query_params,
            path_params: uri.path_params,
            data,
            with_data,
            _t: PhantomData,
        })
    }
}
