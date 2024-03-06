extern crate alohomora;

use alohomora::policy::{AnyPolicy, FrontendPolicy, Policy};
use alohomora_derive::FromBBoxForm;
use std::any::Any;
use rocket::http::Cookie;
use rocket::Request;

pub struct TmpPolicy {}
impl Policy for TmpPolicy {
    fn name(&self) -> String {
        String::from("SamplePolicy")
    }
    fn check(&self, _: &dyn Any) -> bool {
        true
    }
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()>
    where
        Self: Sized,
    {
        todo!()
    }
}
impl FrontendPolicy for TmpPolicy {
    fn from_request(_: &'_ Request<'_>) -> Self {
        TmpPolicy {}
    }
    fn from_cookie<'a, 'r>(
        _name: &str,
        _cookie: &'a Cookie<'static>,
        _request: &'a Request<'r>,
    ) -> Self {
        TmpPolicy {}
    }
}
struct Nested {
    #[allow(dead_code)]
    inner: alohomora::bbox::BBox<String, TmpPolicy>,
}
#[automatically_derived]
const _: () = {
    pub struct FromBBoxFormGeneratedContext<'__a, '__r: '__a> {
        __opts: ::rocket::form::prelude::Options,
        __errors: ::rocket::form::prelude::Errors<'__a>,
        __parent: ::std::option::Option<&'__a ::rocket::form::prelude::Name>,
        __request: ::alohomora::rocket::BBoxRequest<'__a, '__r>,
        #[allow(dead_code)]
        inner: ::std::option::Option<
            <alohomora::bbox::BBox<
                String,
                TmpPolicy,
            > as ::alohomora::rocket::FromBBoxForm<'__a, '__r>>::BBoxContext,
        >,
    }
    impl<'__a, '__r: '__a> FromBBoxFormGeneratedContext<'__a, '__r> {
        fn get_inner_ctx(
            &mut self,
        ) -> &mut <alohomora::bbox::BBox<
            String,
            TmpPolicy,
        > as ::alohomora::rocket::FromBBoxForm<'__a, '__r>>::BBoxContext {
            if let ::std::option::Option::None = self.inner {
                self
                    .inner = ::std::option::Option::Some(
                    <alohomora::bbox::BBox<
                        String,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_init(self.__opts, &self.__request),
                );
            }
            self.inner.as_mut().unwrap()
        }
    }
    impl<'__a, '__r: '__a> ::alohomora::rocket::FromBBoxForm<'__a, '__r> for Nested {
        type BBoxContext = FromBBoxFormGeneratedContext<'__a, '__r>;
        fn bbox_init(
            opts: ::rocket::form::Options,
            request: &::alohomora::rocket::BBoxRequest<'__a, '__r>,
        ) -> Self::BBoxContext {
            Self::BBoxContext {
                __opts: opts,
                __errors: ::rocket::form::prelude::Errors::new(),
                __parent: ::std::option::Option::None,
                __request: request.clone(),
                inner: ::std::option::Option::None,
            }
        }
        fn bbox_push_value(
            ctxt: &mut Self::BBoxContext,
            field: ::alohomora::rocket::BBoxValueField<'__a>,
        ) {
            ctxt.__parent = field.name.parent();
            match field.name.key_lossy().as_str() {
                "inner" => {
                    <alohomora::bbox::BBox<
                        String,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_push_value(ctxt.get_inner_ctx(), field.shift());
                }
                key => {
                    if key != "_method" && ctxt.__opts.strict {
                        ctxt.__errors.push(field.unexpected())
                    }
                }
            }
        }
        #[allow(
            clippy::let_unit_value,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds,
            clippy::used_underscore_binding
        )]
        fn bbox_push_data<'life0, 'async_trait>(
            ctxt: &'life0 mut Self::BBoxContext,
            field: ::alohomora::rocket::BBoxDataField<'__a, '__r>,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = (),
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            '__a: 'async_trait,
            '__r: 'async_trait,
            'life0: 'async_trait,
            Self: 'async_trait,
        {
            Box::pin(async move {
                let ctxt = ctxt;
                let field = field;
                let _: () = {
                    ctxt.__parent = field.name.parent();
                    match field.name.key_lossy().as_str() {
                        "inner" => {
                            let x = alohomora::bbox::BBox::<String, TmpPolicy>::bbox_push_data(ctxt.get_inner_ctx(), field.shift());
                            x.await;
                        }
                        key => {
                            if key != "_method" && ctxt.__opts.strict {
                                ctxt.__errors.push(field.unexpected())
                            }
                        }
                    }
                };
            })
        }
        fn bbox_finalize(
            ctxt: Self::BBoxContext,
        ) -> ::alohomora::rocket::BBoxFormResult<'__a, Self> {
            let mut errors = ctxt.__errors;
            let parent = ctxt.__parent;
            let opts = ctxt.__opts;
            let request = &ctxt.__request;
            let inner = ctxt
                .inner
                .map_or_else(
                    || {
                        <alohomora::bbox::BBox<
                            String,
                            TmpPolicy,
                        > as ::alohomora::rocket::FromBBoxForm<
                            '__a,
                            '__r,
                        >>::bbox_default(opts, request)
                            .ok_or_else(|| {
                                ::rocket::form::prelude::ErrorKind::Missing.into()
                            })
                    },
                    |_ctx| {
                        <alohomora::bbox::BBox<
                            String,
                            TmpPolicy,
                        > as ::alohomora::rocket::FromBBoxForm<
                            '__a,
                            '__r,
                        >>::bbox_finalize(_ctx)
                    },
                )
                .map_err(|e| {
                    let name = ::rocket::form::prelude::NameBuf::from((parent, "inner"));
                    errors.extend(e.with_name(name));
                    ::rocket::form::prelude::Errors::new()
                });
            if errors.is_empty() {
                Ok(Self { inner: inner.unwrap() })
            } else {
                Err(errors)
            }
        }
        fn bbox_push_error(
            ctxt: &mut Self::BBoxContext,
            error: ::rocket::form::Error<'__a>,
        ) {
            ctxt.__errors.push(error);
        }
        fn bbox_default(
            opts: ::rocket::form::Options,
            req: &::alohomora::rocket::BBoxRequest<'__a, '__r>,
        ) -> Option<Self> {
            Self::bbox_finalize(Self::bbox_init(opts, req)).ok()
        }
    }
};
fn main() {}
