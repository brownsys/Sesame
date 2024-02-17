extern crate proc_macro2;
extern crate quote;
extern crate syn;

use proc_macro2::{Ident, Span, TokenStream};
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{
    Data, DataStruct, DeriveInput, Field, Fields, GenericParam, Generics, Lifetime, LifetimeParam,
    Type,
};

pub fn context_generics(mut generics: Generics) -> Generics {
    generics.params.insert(
        0,
        GenericParam::Lifetime(LifetimeParam {
            attrs: Vec::new(),
            lifetime: Lifetime {
                apostrophe: Span::call_site(),
                ident: Ident::new("__f", Span::call_site()),
            },
            colon_token: None,
            bounds: Punctuated::new(),
        }),
    );
    generics
}

pub fn cast_field_types(fields: &Punctuated<Field, Comma>) -> Vec<TokenStream> {
    fields
        .iter()
        .map(|field| {
            let field_type = &field.ty;
            quote! {
              <#field_type as ::bbox::rocket::FromBBoxForm<'__f>>
            }
        })
        .collect()
}

pub fn generate_context(
    generics: &Generics,
    fields: &Punctuated<Field, Comma>,
    types: &Vec<TokenStream>,
) -> TokenStream {
    // The context of every field in the source struct becomes a field with the same name
    // in the generated context struct.
    let context_fields = fields.iter().zip(types.iter()).map(|(field, ty)| {
        let mut result = field.clone();
        result.ty = Type::Verbatim(quote! {
          ::std::option::Option<#ty::BBoxContext>,
        });
        result
    });

    // Getters for every field context.
    let getters = fields.iter().zip(types.iter()).map(|(field, ty)| {
    let ident = field.ident.as_ref().unwrap();
    let function_name = Ident::new(&format!("get_{}_ctx", ident), ident.span());
    quote! {
      fn #function_name (&mut self, opts: ::rocket::form::prelude::Options) -> &mut #ty::BBoxContext {
        if let ::std::option::Option::None = self.#ident {
          self.#ident = ::std::option::Option::Some(#ty::bbox_init(opts));
        }
        self.#ident.as_mut().unwrap()
      }
    }
  });

    // Generated context struct must declare the same genercis.
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    // Generate the struct.
    quote! {
      pub struct FromBBoxFormGeneratedContext #generics {
        __opts: ::rocket::form::prelude::Options,
        __errors: ::rocket::form::prelude::Errors<'__f>,
        __parent: ::std::option::Option<&'__f ::rocket::form::prelude::Name>,
        #(#context_fields)*
      }
      impl #impl_generics FromBBoxFormGeneratedContext #ty_generics #where_clause {
        #(#getters)*
      }
    }
}

// Generate cases for push_value and push_data based on matching the current
// form key string with each field name.
pub fn generate_push_value_cases(
    fields: &Vec<Ident>,
    types: &Vec<TokenStream>,
) -> Vec<TokenStream> {
    fields
        .iter()
        .zip(types.iter())
        .map(|(field, ty)| {
            let getter = Ident::new(&format!("get_{}_ctx", field), Span::call_site());
            let name = field.to_string();
            quote! {
              #name => {
                let _ctx = ctxt.#getter(ctxt.__opts);
                #ty::bbox_push_value(_ctx, field.shift());
              },
            }
        })
        .collect()
}
pub fn generate_push_data_cases(fields: &Vec<Ident>, types: &Vec<TokenStream>) -> Vec<TokenStream> {
    fields
        .iter()
        .zip(types.iter())
        .map(|(field, ty)| {
            let getter = Ident::new(&format!("get_{}_ctx", field), Span::call_site());
            let name = field.to_string();
            quote! {
              #name => {
                let _ctx = ctxt.#getter(ctxt.__opts);
                let future = #ty::bbox_push_data(_ctx, field.shift());
                future.await;
              },
            }
        })
        .collect()
}

// Generate code to finialize the context of every field and aggregate any errors.
pub fn generate_finalize_cases(fields: &Vec<Ident>, types: &Vec<TokenStream>) -> Vec<TokenStream> {
    fields.iter().zip(types.iter()).map(|(field, ty)| {
    let name = field.to_string();
    quote! {
      let #field = ctxt.#field.map_or_else(
        || {
          #ty::bbox_default(opts).ok_or_else(|| ::rocket::form::prelude::ErrorKind::Missing.into())
        },
        |_ctx| {
          #ty::bbox_finalize(_ctx)
        }
      ).map_err(|e| {
        let name = ::rocket::form::prelude::NameBuf::from((parent, #name));
        errors.extend(e.with_name(name));
        ::rocket::form::prelude::Errors::new()
      });
    }
  }).collect()
}

pub fn derive_from_bbox_form_impl(input: DeriveInput) -> TokenStream {
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

    // Get all the field names.
    let fields_idents: Vec<Ident> = fields
        .iter()
        .map(|field| field.ident.clone().unwrap())
        .collect();
    let casted_fields_types = cast_field_types(&fields);

    // The context struct type def.
    let context_generics = context_generics(input.generics.clone());
    let context = generate_context(&context_generics, &fields, &casted_fields_types);

    // Split generics to be compatible with impl blocks.
    let (impl_generics, ctx_ty_generics, _) = context_generics.split_for_impl();
    let (_, ty_generics, where_clause) = input.generics.split_for_impl();

    // Generate cases for push_value and push_data for all the fields.
    let push_value_cases = generate_push_value_cases(&fields_idents, &casted_fields_types);
    let push_data_cases = generate_push_data_cases(&fields_idents, &casted_fields_types);

    // Generate a call to bbox_finalize(...) for every field.
    let finalize_cases = generate_finalize_cases(&fields_idents, &casted_fields_types);

    // Impl trait.
    quote! {
      #[automatically_derived]
      const _: () = {
        #context

        #[::rocket::async_trait]
        impl #impl_generics ::bbox::rocket::FromBBoxForm<'__f> for #input_name #ty_generics #where_clause {
          type BBoxContext = FromBBoxFormGeneratedContext #ctx_ty_generics;

          // Required methods
          fn bbox_init(opts: ::rocket::form::Options) -> Self::BBoxContext {
            Self::BBoxContext {
              __opts: opts,
              __errors: ::rocket::form::prelude::Errors::new(),
              __parent: ::std::option::Option::None,
              // TODO(babman): default values?
              #( #fields_idents: ::std::option::Option::None, )*
            }
          }

          // Push data for url_encoded bodies.
          fn bbox_push_value(ctxt: &mut Self::BBoxContext, field: ::bbox::rocket::BBoxValueField<'__f>) {
            ctxt.__parent = field.name.parent();
            match field.name.key_lossy().as_str() {
              #(#push_value_cases)*
              // must be last case.
              key => {
                if key != "_method" && ctxt.__opts.strict {
                  ctxt.__errors.push(field.unexpected())
                }
              },
            }
          }

          // Push data for multipart bodies.
          async fn bbox_push_data(
            ctxt: &mut Self::BBoxContext,
            field: ::bbox::rocket::BBoxDataField<'__f, '_>,
          ) {
            ctxt.__parent = field.name.parent();
            match field.name.key_lossy().as_str() {
              #(#push_data_cases)*
              // must be last case.
              key => {
                if key != "_method" && ctxt.__opts.strict {
                  ctxt.__errors.push(field.unexpected())
                }
              },
            }
          }

          // Finalize.
          fn bbox_finalize(ctxt: Self::BBoxContext) -> ::bbox::rocket::BBoxFormResult<'__f, Self> {
            let mut errors = ctxt.__errors;
            let parent = ctxt.__parent;
            let opts = ctxt.__opts;

            // Will populate errors with any existing erorrs and create a variable
            // containing the result of every field with the same name.
            #(#finalize_cases)*

            // Return result or errors.
            if errors.is_empty() {
              Ok(Self {
                #( #fields_idents: #fields_idents.unwrap(), )*
              })
            } else {
              Err(errors)
            }
          }

          // Provided methods
          fn bbox_push_error(ctxt: &mut Self::BBoxContext, error: ::rocket::form::Error<'__f>) {
            ctxt.__errors.push(error);
          }
          fn bbox_default(opts: ::rocket::form::Options) -> Option<Self> {
            Self::bbox_finalize(Self::bbox_init(opts)).ok()
          }
        }
      };
    }
}
