extern crate proc_macro2;
extern crate quote;
extern crate syn;

use std::collections::HashMap;
use std::option::Option;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::{FnArg, ItemFn, Pat, Type};

use crate::route::{RouteArgs, RouteType};

#[derive(PartialEq, Hash, Eq)]
struct Parameter {
    name: String,
}
impl Parameter {
    pub fn new(name: String) -> Self {
        Parameter { name }
    }
    pub fn to_string(&self) -> &str {
        &self.name
    }
    pub fn to_ident(&self) -> Ident {
        Ident::new(&self.name, Span::call_site())
    }
}

impl PartialEq<str> for Parameter {
    fn eq(&self, other: &str) -> bool {
        self.name == other
    }
}

type PathParam = (Parameter, usize);

// Types of parameters.
enum ParamClass {
    Data,      // Use FromBBoxData.
    Query,     // Use FromBBoxForm.
    Path,      // Use FromBBoxParam.
    DataGuard, // Use FromBBoxRequest.
    DataGuardWithData, // Use FromBBoxRequestAndData
}

// Easy to use format of the macro inputs.
struct RouteAttribute {
    pub method: Ident,
    // function name.
    pub func_name: Option<Ident>,
    // post request data parameter name.
    pub data: Option<Parameter>,
    // anything that needs FromBBoxRequestAndData (usually the context).
    pub with_data: Option<Parameter>,
    // all get parameters in the query (?<x>)
    pub query_params: Vec<Parameter>,
    // all uri path parameters (/<x>/...)
    pub path_params: Vec<PathParam>,
    // other FromBBoxRequest guards.
    pub guards: Vec<Parameter>,
    // uri.
    pub uri: String,
    // maps parameter name to its declared type.
    pub types: HashMap<Parameter, Type>,
    // the handler's arguments in order.
    pub args: Vec<Parameter>,
}
impl RouteAttribute {
    pub fn new<T: RouteType>(args: RouteArgs<T>) -> Self {
        Self {
            method: args.method,
            func_name: None,
            data: args.data.map(Parameter::new),
            with_data: args.with_data.map(Parameter::new),
            query_params: args.query_params.into_iter().map(Parameter::new).collect(),
            path_params: args
                .path_params
                .into_iter()
                .map(|(p, i)| (Parameter::new(p), i))
                .collect(),
            guards: Vec::new(),
            uri: args.uri,
            types: HashMap::new(),
            args: Vec::new(),
        }
    }

    // Given a parameter name, check what type of parameter it is.
    pub fn class_of(&self, name: &str) -> Option<ParamClass> {
        if self.query_params.iter().any(|p| p == name) {
            Some(ParamClass::Query)
        } else if self.path_params.iter().any(|(p, _)| p == name) {
            Some(ParamClass::Path)
        } else if self.guards.iter().any(|p| p == name) {
            Some(ParamClass::DataGuard)
        } else {
            match &self.data {
                Some(p) if p == name => Some(ParamClass::Data),
                _ => match &self.with_data {
                  Some(p) if p == name => Some(ParamClass::DataGuardWithData),
                  _ => None,
                },
            }
        }
    }

    pub fn call_function(&self) -> TokenStream {
        let func = self.func_name.as_ref().unwrap();
        let args = self.args.iter().map(Parameter::to_ident);
        quote! {
          #func ( #(#args),* )
        }
    }

    pub fn param_count(&self) -> usize {
        let mut result = if self.data.is_some() { 1 } else { 0 };
        result += if self.with_data.is_some() { 1 } else { 0 };
        result += self.query_params.len() + self.path_params.len();
        result += self.guards.len();
        result
    }
}

pub fn route_impl<T: RouteType>(args: RouteArgs<T>, input: ItemFn) -> TokenStream {
    let mut args = RouteAttribute::new(args);

    // function cannot be generic.
    let sig = input.sig;
    args.func_name = Some(sig.ident);
    if sig.generics.params.len() > 0 {
        panic!("Cannot apply route macros to a generic fn");
    }

    // Go over arguments.
    for arg in sig.inputs.iter() {
        // Extract argument name.
        let arg_name = match arg {
            FnArg::Receiver(_) => panic!("function cannot take self"),
            FnArg::Typed(arg) => match *arg.pat {
                Pat::Ident(ref pat) => &pat.ident,
                _ => panic!("bad argument name"),
            },
        }
        .to_string();
        let arg_type = match arg {
            FnArg::Receiver(_) => panic!("function cannot take self"),
            FnArg::Typed(arg) => (*arg.ty).clone(),
        };

        // Record argument in order.
        args.args.push(Parameter::new(arg_name.clone()));

        // Find its class.
        if args.class_of(&arg_name).is_none() {
            args.guards.push(Parameter::new(arg_name.clone()));
        }

        // Record its type.
        args.types.insert(Parameter::new(arg_name), arg_type);
    }

    // Make sure count is correct.
    if args.param_count() != args.args.len() {
        panic!("Incorrect argument count");
    }

    // Now, we start generating code.
    let method = &args.method;
    let fn_name = args.func_name.as_ref().unwrap();
    let uri = &args.uri;
    let fn_call = args.call_function();

    // Do Path parameters first.
    let path_params = args.path_params.iter().map(|(param, idx)| {
        let ident = param.to_ident();
        quote! {
          let #ident = match _request.param(#idx) {
            ::std::option::Option::Some(_d) => match _d {
              ::std::result::Result::Ok(d) => d,
              ::std::result::Result::Err(_) => {
                return ::alohomora::rocket::BBoxResponseOutcome::Forward(_data);
              },
            },
            ::std::option::Option::None => {
              return ::alohomora::rocket::BBoxResponseOutcome::Forward(_data);
            },
          };
        }
    });

    // Do data guards.
    let data_guards = args.guards.iter().map(|param| {
      let ident = param.to_ident();
      let ty = args.types.get(param).unwrap();

      quote! {
        let #ident = match <#ty as ::alohomora::rocket::FromBBoxRequest>::from_bbox_request(_request).await {
          ::alohomora::rocket::BBoxRequestOutcome::Success(_d) => _d,
          ::alohomora::rocket::BBoxRequestOutcome::Failure((_s, _e)) => {
            return ::alohomora::rocket::BBoxResponseOutcome::Failure(_s);
          },
          ::alohomora::rocket::BBoxRequestOutcome::Forward(_) => {
            return ::alohomora::rocket::BBoxResponseOutcome::Forward(_data);
          },
        };
      }
    });

    // Do post data.
    let mut with_data = quote!{};
    let post_data = match args.data.as_ref() {
        None => quote! {},
        Some(data) => {
            let ident = data.to_ident();
            let data_ty = args.types.get(data).unwrap();
            let post_data = quote! {
              let #ident = match <#data_ty as ::alohomora::rocket::FromBBoxData>::from_data(_request, _data).await {
                ::alohomora::rocket::BBoxDataOutcome::Success(_d) => _d,
                ::alohomora::rocket::BBoxDataOutcome::Failure((_s, _e)) => {
                  return ::alohomora::rocket::BBoxResponseOutcome::Failure(_s);
                },
                ::alohomora::rocket::BBoxDataOutcome::Forward(_f) => {
                  return ::alohomora::rocket::BBoxResponseOutcome::Forward(_f);
                },
              };
            };

            // Pass the form data parameter to the context if it needs from_bbox_request_and_data.
            with_data = match args.with_data.as_ref() {
                None => quote! {},
                Some(with_data) => {
                    let with_data_ident = with_data.to_ident();
                    let ty = args.types.get(with_data).unwrap();
                    quote! {
                      let #with_data_ident = match <#ty as ::alohomora::rocket::FromBBoxRequestAndData<#data_ty>>::from_bbox_request_and_data(_request, &#ident).await {
                        ::alohomora::rocket::BBoxRequestOutcome::Success(_d) => _d,
                        ::alohomora::rocket::BBoxRequestOutcome::Failure((_s, _e)) => {
                          return ::alohomora::rocket::BBoxResponseOutcome::Failure(_s);
                        },
                        ::alohomora::rocket::BBoxRequestOutcome::Forward(_f) => {
                          panic!("With_data member forwarded but data is consumed");
                        },
                      };
                    }
                }
            };

            post_data
        }
    };

    // Parse all get parameters in one shot.
    let query_idents = args
        .query_params
        .iter()
        .map(|param| param.to_ident())
        .collect::<Vec<_>>();
    let query_strings = args
        .query_params
        .iter()
        .map(|param| param.to_string())
        .collect::<Vec<_>>();
    let query_casted_types = args
        .query_params
        .iter()
        .map(|param| {
            let ty = args.types.get(param).unwrap();
            quote! {
              <#ty as ::alohomora::rocket::FromBBoxForm>
            }
        })
        .collect::<Vec<_>>();

    let get_params = quote! {
      let mut _errors = ::rocket::form::prelude::Errors::new();

      // initialize.
      let _opts = ::rocket::form::prelude::Options::Lenient;
      #(let mut #query_idents = #query_casted_types::bbox_init(_opts);)*

      // push.
      for _field in _request.query_fields() {
        match _field.name.key_lossy().as_str() {
          #(#query_strings => #query_casted_types::bbox_push_value(&mut #query_idents, _field.shift(), _request),)*
          _ => {},
        }
      }

      // finalize.
      #(let #query_idents = match #query_casted_types::bbox_finalize(#query_idents) {
        ::std::result::Result::Ok(_v) => ::std::option::Option::Some(_v),
        ::std::result::Result::Err(_err) => {
          _errors.extend(_err.with_name(::rocket::form::prelude::NameView::new(#query_strings)));
          ::std::option::Option::None
        },
      };)*

      // handle any errors.
      if !_errors.is_empty() {
        return ::alohomora::rocket::BBoxResponseOutcome::Forward(_data);
      }
      #(let #query_idents = #query_idents.unwrap();)*
    };

    // If the function is async, we should await on the result.
    let res_await = if sig.asyncness.is_some() {
        quote! {
            let _res = _res.await;
        }
    } else {
        quote! {}
    };

    quote! {
      #[allow(non_camel_case_types)]
      pub struct #fn_name {}
      impl #fn_name {
        pub async fn lambda<'a, 'r>(_request: ::alohomora::rocket::BBoxRequest<'a, 'r>, _data: ::alohomora::rocket::BBoxData<'a>) -> ::alohomora::rocket::BBoxResponseOutcome<'a> {
          // Path parameters.
          #(#path_params)*

          // Get parameters.
          #get_params

          // Guards.
          #(#data_guards)*

          // POST data (if any).
          #post_data

          // with_data (context, if any).
          #with_data

          // invoke with result.
          let _res = #fn_call;

          // await on response if handler is async.
          #res_await

          // done!
          ::alohomora::rocket::BBoxResponseOutcome::from(_request, _res)
        }

        pub fn info() -> ::alohomora::rocket::BBoxRouteInfo {
          ::alohomora::rocket::BBoxRouteInfo {
            method: ::rocket::http::Method::#method,
            uri: #uri,
            bbox_handler: |request, data| {
              ::std::boxed::Box::pin(Self::lambda(request, data))
            },
          }
        }
      }
    }
}
