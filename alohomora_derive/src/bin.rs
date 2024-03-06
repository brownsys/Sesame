extern crate alohomora;
use alohomora::context::Context;
use alohomora::policy::{AnyPolicy, FrontendPolicy, Policy};
use alohomora_derive::{route, routes, FromBBoxForm};
use std::any::Any;
use rocket::http::Cookie;
use rocket::{futures, Request};
use alohomora::pcr::PrivacyCriticalRegion;
use alohomora::rocket::{BBoxData, BBoxRequest, BBoxResponseOutcome};
use alohomora::unbox::unbox;
pub struct TmpPolicy {}
#[automatically_derived]
impl ::core::clone::Clone for TmpPolicy {
    #[inline]
    fn clone(&self) -> TmpPolicy {
        TmpPolicy {}
    }
}
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
struct Simple {
    f1: alohomora::bbox::BBox<String, TmpPolicy>,
    f3: alohomora::bbox::BBox<u8, TmpPolicy>,
}
const _: () = {
    pub struct FromBBoxFormGeneratedContext<'__a, '__r: '__a> {
        __opts: ::rocket::form::prelude::Options,
        __errors: ::rocket::form::prelude::Errors<'__a>,
        __parent: ::std::option::Option<&'__a ::rocket::form::prelude::Name>,
        __request: ::alohomora::rocket::BBoxRequest<'__a, '__r>,
        f1: ::std::option::Option<
            <alohomora::bbox::BBox<
                String,
                TmpPolicy,
            > as ::alohomora::rocket::FromBBoxForm<'__a, '__r>>::BBoxContext,
        >,
        f3: ::std::option::Option<
            <alohomora::bbox::BBox<
                u8,
                TmpPolicy,
            > as ::alohomora::rocket::FromBBoxForm<'__a, '__r>>::BBoxContext,
        >,
    }
    impl<'__a, '__r: '__a> FromBBoxFormGeneratedContext<'__a, '__r> {
        fn get_f1_ctx(
            &mut self,
        ) -> &mut <alohomora::bbox::BBox<
            String,
            TmpPolicy,
        > as ::alohomora::rocket::FromBBoxForm<'__a, '__r>>::BBoxContext {
            if let ::std::option::Option::None = self.f1 {
                self
                    .f1 = ::std::option::Option::Some(
                    <alohomora::bbox::BBox<
                        String,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_init(self.__opts, &self.__request),
                );
            }
            self.f1.as_mut().unwrap()
        }
        fn get_f3_ctx(
            &mut self,
        ) -> &mut <alohomora::bbox::BBox<
            u8,
            TmpPolicy,
        > as ::alohomora::rocket::FromBBoxForm<'__a, '__r>>::BBoxContext {
            if let ::std::option::Option::None = self.f3 {
                self
                    .f3 = ::std::option::Option::Some(
                    <alohomora::bbox::BBox<
                        u8,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_init(self.__opts, &self.__request),
                );
            }
            self.f3.as_mut().unwrap()
        }
    }
    #[automatically_derived]
    impl<'__a, '__r: '__a> ::alohomora::rocket::FromBBoxForm<'__a, '__r> for Simple {
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
                f1: ::std::option::Option::None,
                f3: ::std::option::Option::None,
            }
        }
        fn bbox_push_value(
            ctxt: &mut Self::BBoxContext,
            field: ::alohomora::rocket::BBoxValueField<'__a>,
        ) {
            ctxt.__parent = field.name.parent();
            match field.name.key_lossy().as_str() {
                "f1" => {
                    <alohomora::bbox::BBox<
                        String,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_push_value(ctxt.get_f1_ctx(), field.shift());
                }
                "f3" => {
                    <alohomora::bbox::BBox<
                        u8,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_push_value(ctxt.get_f3_ctx(), field.shift());
                }
                key => {
                    if key != "_method" && ctxt.__opts.strict {
                        ctxt.__errors.push(field.unexpected())
                    }
                }
            }
        }
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
            ctxt.__parent = field.name.parent();
            match field.name.key_lossy().as_str() {
                "f1" => {
                    <alohomora::bbox::BBox<
                        String,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_push_data(ctxt.get_f1_ctx(), field.shift())
                }
                "f3" => {
                    <alohomora::bbox::BBox<
                        u8,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_push_data(ctxt.get_f3_ctx(), field.shift())
                }
                key => {
                    if key != "_method" && ctxt.__opts.strict {
                        ctxt.__errors.push(field.unexpected())
                    }
                    Box::pin(::std::future::ready(()))
                }
            }
        }
        fn bbox_finalize(
            ctxt: Self::BBoxContext,
        ) -> ::alohomora::rocket::BBoxFormResult<'__a, Self> {
            let mut errors = ctxt.__errors;
            let parent = ctxt.__parent;
            let opts = ctxt.__opts;
            let request = &ctxt.__request;
            let f1 = ctxt
                .f1
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
                    let name = ::rocket::form::prelude::NameBuf::from((parent, "f1"));
                    errors.extend(e.with_name(name));
                    ::rocket::form::prelude::Errors::new()
                });
            let f3 = ctxt
                .f3
                .map_or_else(
                    || {
                        <alohomora::bbox::BBox<
                            u8,
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
                            u8,
                            TmpPolicy,
                        > as ::alohomora::rocket::FromBBoxForm<
                            '__a,
                            '__r,
                        >>::bbox_finalize(_ctx)
                    },
                )
                .map_err(|e| {
                    let name = ::rocket::form::prelude::NameBuf::from((parent, "f3"));
                    errors.extend(e.with_name(name));
                    ::rocket::form::prelude::Errors::new()
                });
            if errors.is_empty() {
                Ok(Self {
                    f1: f1.unwrap(),
                    f3: f3.unwrap(),
                })
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
struct Config {
    pub debug_mode: bool,
    pub admins: std::collections::HashSet<String>,
}
impl Config {
    pub fn new(admin: &str) -> Self {
        let mut c = Config {
            debug_mode: false,
            admins: std::collections::HashSet::new(),
        };
        c.admins.insert(String::from(admin));
        c
    }
}
struct MyGuard {
    pub value: String,
}
impl<'a, 'r> alohomora::rocket::FromBBoxRequest<'a, 'r> for MyGuard {
    type BBoxError = &'static str;
    #[allow(
        clippy::let_unit_value,
        clippy::type_complexity,
        clippy::type_repetition_in_bounds,
        clippy::used_underscore_binding
    )]
    fn from_bbox_request<'async_trait>(
        _request: &'a alohomora::rocket::BBoxRequest<'a, 'r>,
    ) -> ::core::pin::Pin<
        Box<
            dyn ::core::future::Future<
                Output = alohomora::rocket::BBoxRequestOutcome<Self, Self::BBoxError>,
            > + ::core::marker::Send + 'async_trait,
        >,
    >
    where
        'a: 'async_trait,
        'r: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            if let ::core::option::Option::Some(__ret)
                = ::core::option::Option::None::<
                    alohomora::rocket::BBoxRequestOutcome<Self, Self::BBoxError>,
                > {
                return __ret;
            }
            let _request = _request;
            let __ret: alohomora::rocket::BBoxRequestOutcome<Self, Self::BBoxError> = {
                let guard = MyGuard {
                    value: String::from("ok"),
                };
                alohomora::rocket::BBoxRequestOutcome::Success(guard)
            };
            #[allow(unreachable_code)] __ret
        })
    }
}
struct Dog {
    name: alohomora::bbox::BBox<String, TmpPolicy>,
    age: alohomora::bbox::BBox<usize, TmpPolicy>,
}
const _: () = {
    pub struct FromBBoxFormGeneratedContext<'__a, '__r: '__a> {
        __opts: ::rocket::form::prelude::Options,
        __errors: ::rocket::form::prelude::Errors<'__a>,
        __parent: ::std::option::Option<&'__a ::rocket::form::prelude::Name>,
        __request: ::alohomora::rocket::BBoxRequest<'__a, '__r>,
        name: ::std::option::Option<
            <alohomora::bbox::BBox<
                String,
                TmpPolicy,
            > as ::alohomora::rocket::FromBBoxForm<'__a, '__r>>::BBoxContext,
        >,
        age: ::std::option::Option<
            <alohomora::bbox::BBox<
                usize,
                TmpPolicy,
            > as ::alohomora::rocket::FromBBoxForm<'__a, '__r>>::BBoxContext,
        >,
    }
    impl<'__a, '__r: '__a> FromBBoxFormGeneratedContext<'__a, '__r> {
        fn get_name_ctx(
            &mut self,
        ) -> &mut <alohomora::bbox::BBox<
            String,
            TmpPolicy,
        > as ::alohomora::rocket::FromBBoxForm<'__a, '__r>>::BBoxContext {
            if let ::std::option::Option::None = self.name {
                self
                    .name = ::std::option::Option::Some(
                    <alohomora::bbox::BBox<
                        String,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_init(self.__opts, &self.__request),
                );
            }
            self.name.as_mut().unwrap()
        }
        fn get_age_ctx(
            &mut self,
        ) -> &mut <alohomora::bbox::BBox<
            usize,
            TmpPolicy,
        > as ::alohomora::rocket::FromBBoxForm<'__a, '__r>>::BBoxContext {
            if let ::std::option::Option::None = self.age {
                self
                    .age = ::std::option::Option::Some(
                    <alohomora::bbox::BBox<
                        usize,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_init(self.__opts, &self.__request),
                );
            }
            self.age.as_mut().unwrap()
        }
    }
    #[automatically_derived]
    impl<'__a, '__r: '__a> ::alohomora::rocket::FromBBoxForm<'__a, '__r> for Dog {
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
                name: ::std::option::Option::None,
                age: ::std::option::Option::None,
            }
        }
        fn bbox_push_value(
            ctxt: &mut Self::BBoxContext,
            field: ::alohomora::rocket::BBoxValueField<'__a>,
        ) {
            ctxt.__parent = field.name.parent();
            match field.name.key_lossy().as_str() {
                "name" => {
                    <alohomora::bbox::BBox<
                        String,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_push_value(ctxt.get_name_ctx(), field.shift());
                }
                "age" => {
                    <alohomora::bbox::BBox<
                        usize,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_push_value(ctxt.get_age_ctx(), field.shift());
                }
                key => {
                    if key != "_method" && ctxt.__opts.strict {
                        ctxt.__errors.push(field.unexpected())
                    }
                }
            }
        }
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
            ctxt.__parent = field.name.parent();
            match field.name.key_lossy().as_str() {
                "name" => {
                    <alohomora::bbox::BBox<
                        String,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_push_data(ctxt.get_name_ctx(), field.shift())
                }
                "age" => {
                    <alohomora::bbox::BBox<
                        usize,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm<
                        '__a,
                        '__r,
                    >>::bbox_push_data(ctxt.get_age_ctx(), field.shift())
                }
                key => {
                    if key != "_method" && ctxt.__opts.strict {
                        ctxt.__errors.push(field.unexpected())
                    }
                    Box::pin(::std::future::ready(()))
                }
            }
        }
        fn bbox_finalize(
            ctxt: Self::BBoxContext,
        ) -> ::alohomora::rocket::BBoxFormResult<'__a, Self> {
            let mut errors = ctxt.__errors;
            let parent = ctxt.__parent;
            let opts = ctxt.__opts;
            let request = &ctxt.__request;
            let name = ctxt
                .name
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
                    let name = ::rocket::form::prelude::NameBuf::from((parent, "name"));
                    errors.extend(e.with_name(name));
                    ::rocket::form::prelude::Errors::new()
                });
            let age = ctxt
                .age
                .map_or_else(
                    || {
                        <alohomora::bbox::BBox<
                            usize,
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
                            usize,
                            TmpPolicy,
                        > as ::alohomora::rocket::FromBBoxForm<
                            '__a,
                            '__r,
                        >>::bbox_finalize(_ctx)
                    },
                )
                .map_err(|e| {
                    let name = ::rocket::form::prelude::NameBuf::from((parent, "age"));
                    errors.extend(e.with_name(name));
                    ::rocket::form::prelude::Errors::new()
                });
            if errors.is_empty() {
                Ok(Self {
                    name: name.unwrap(),
                    age: age.unwrap(),
                })
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
fn my_route(
    guard: MyGuard,
    num: alohomora::bbox::BBox<u8, TmpPolicy>,
    data: alohomora::rocket::BBoxForm<Simple>,
    config: &rocket::State<Config>,
    a: alohomora::bbox::BBox<String, TmpPolicy>,
    dog: Dog,
) -> alohomora::rocket::BBoxRedirect {
    match (&guard.value, &"ok") {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                todo!()
            }
        }
    };
    match (&config.debug_mode, &false) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                todo!()
            }
        }
    };
    match (&config.admins.len(), &1) {
        (left_val, right_val) => {
            if !(*left_val == *right_val) {
                todo!()
            }
        }
    };
    if !config.admins.contains("test@email.com") {
        todo!()
    }
    let context = Context::new(Option::None::<()>, String::from(""), ());
    let result = unbox(
        (num.clone(), a.clone(), data.f1.clone(), data.f3.clone(), dog.name, dog.age),
        &context,
        PrivacyCriticalRegion::new(|(num, a, f1, f3, name, age), _| {
            match (&&f1, &"str1") {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        todo!()
                    }
                }
            };
            match (&f3, &10) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        todo!()
                    }
                }
            };
            match (&num, &5) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        todo!()
                    }
                }
            };
            match (&&a, &"apple") {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        todo!()
                    }
                }
            };
            match (&&name, &"Max") {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        todo!()
                    }
                }
            };
            match (&age, &10) {
                (left_val, right_val) => {
                    if !(*left_val == *right_val) {
                        todo!()
                    }
                }
            };
        }),
        (),
    );
    result.unwrap();
    alohomora::rocket::BBoxRedirect::to("/page/{}/{}/{}/{}", (&a, &num, &"test", &10))
}
#[allow(non_camel_case_types)]
pub struct my_route {}
impl my_route {
    pub async fn lambda<'a>(
        _request: ::alohomora::rocket::BBoxRequest<'a, '_>,
        _data: ::alohomora::rocket::BBoxData<'a>,
    ) -> ::alohomora::rocket::BBoxResponseOutcome<'a> {
        let forward = BBoxResponseOutcome::Forward(_data);
        return forward;

        let num = match _request.param(1usize) {
            ::std::option::Option::Some(_d) => {
                match _d {
                    ::std::result::Result::Ok(d) => d,
                    ::std::result::Result::Err(_) => {
                        // return ::alohomora::rocket::BBoxResponseOutcome::Forward(_data);
                        return todo!();
                    }
                }
            }
            ::std::option::Option::None => {
                //return ::alohomora::rocket::BBoxResponseOutcome::Forward(_data);
                return todo!();
            }
        };
        let mut _errors = ::rocket::form::prelude::Errors::new();
        let opts = ::rocket::form::prelude::Options::Lenient;
        let mut dog = <Dog as ::alohomora::rocket::FromBBoxForm>::bbox_init(
            opts,
            &_request,
        );
        let mut a = <alohomora::bbox::BBox<
            String,
            TmpPolicy,
        > as ::alohomora::rocket::FromBBoxForm>::bbox_init(opts, &_request);
        for field in _request.query_fields() {
            match field.name.key_lossy().as_str() {
                "dog" => {
                    <Dog as ::alohomora::rocket::FromBBoxForm>::bbox_push_value(
                        &mut dog,
                        field.shift(),
                    )
                }
                "a" => {
                    <alohomora::bbox::BBox<
                        String,
                        TmpPolicy,
                    > as ::alohomora::rocket::FromBBoxForm>::bbox_push_value(
                        &mut a,
                        field.shift(),
                    )
                }
                _ => {}
            }
        }
        let dog = match <Dog as ::alohomora::rocket::FromBBoxForm>::bbox_finalize(dog) {
            ::std::result::Result::Ok(_v) => ::std::option::Option::Some(_v),
            ::std::result::Result::Err(_err) => {
                _errors
                    .extend(
                        _err.with_name(::rocket::form::prelude::NameView::new("dog")),
                    );
                ::std::option::Option::None
            }
        };
        let a = match <alohomora::bbox::BBox<
            String,
            TmpPolicy,
        > as ::alohomora::rocket::FromBBoxForm>::bbox_finalize(a) {
            ::std::result::Result::Ok(_v) => ::std::option::Option::Some(_v),
            ::std::result::Result::Err(_err) => {
                _errors
                    .extend(_err.with_name(::rocket::form::prelude::NameView::new("a")));
                ::std::option::Option::None
            }
        };
        if !_errors.is_empty() {
            return ::alohomora::rocket::BBoxResponseOutcome::Forward(_data);
        }
        let dog = dog.unwrap();
        let a = a.unwrap();
        let guard = match <MyGuard as ::alohomora::rocket::FromBBoxRequest>::from_bbox_request(
                &_request,
            )
            .await
        {
            ::alohomora::rocket::BBoxRequestOutcome::Success(_d) => _d,
            ::alohomora::rocket::BBoxRequestOutcome::Failure((_s, _e)) => {
                return ::alohomora::rocket::BBoxResponseOutcome::Failure(_s);
            }
            ::alohomora::rocket::BBoxRequestOutcome::Forward(_) => {
                return ::alohomora::rocket::BBoxResponseOutcome::Forward(_data);
            }
        };
        let config = match <&rocket::State<
            Config,
        > as ::alohomora::rocket::FromBBoxRequest>::from_bbox_request(&_request)
            .await
        {
            ::alohomora::rocket::BBoxRequestOutcome::Success(_d) => _d,
            ::alohomora::rocket::BBoxRequestOutcome::Failure((_s, _e)) => {
                return ::alohomora::rocket::BBoxResponseOutcome::Failure(_s);
            }
            ::alohomora::rocket::BBoxRequestOutcome::Forward(_) => {
                return ::alohomora::rocket::BBoxResponseOutcome::Forward(_data);
            }
        };
        let data = match <alohomora::rocket::BBoxForm<
            Simple,
        > as ::alohomora::rocket::FromBBoxData>::from_data(&_request, _data)
            .await
        {
            ::alohomora::rocket::BBoxDataOutcome::Success(_d) => _d,
            ::alohomora::rocket::BBoxDataOutcome::Failure((_s, _e)) => {
                return ::alohomora::rocket::BBoxResponseOutcome::Failure(_s);
            }
            ::alohomora::rocket::BBoxDataOutcome::Forward(_f) => {
                return ::alohomora::rocket::BBoxResponseOutcome::Forward(_f);
            }
        };
        let res = my_route(guard, num, data, config, a, dog);
        ::alohomora::rocket::BBoxResponseOutcome::from(&_request, res)
    }
    pub fn info() -> ::alohomora::rocket::BBoxRouteInfo {
        fn lam<'a>(request: BBoxRequest<'a, '_>, data: BBoxData<'a>) -> futures::future::BoxFuture<'a, BBoxResponseOutcome<'a>> {
            let x = my_route::lambda(request, data);
            ::std::boxed::Box::pin(x)
        }

        ::alohomora::rocket::BBoxRouteInfo {
            method: ::rocket::http::Method::Post,
            uri: "/route/<num>?<dog>&<a>",
            bbox_handler: lam,
        }
    }
}

pub fn main() -> () {
    let _rocket = alohomora::rocket::BBoxRocket::<::rocket::Build>::build()
        .manage(Config::new("test@email.com"))
        .mount("/test", routes![my_route]);
}
