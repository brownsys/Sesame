#![feature(prelude_import)]
#[prelude_import]
use std::prelude::rust_2018::*;
#[macro_use]
extern crate std;
use rocket::{FromForm, get, routes};
pub struct Dog {
    a: String,
    b: u32,
}
#[allow(non_snake_case)]
const _: () = {
    /// Rocket generated FormForm context.
    #[doc(hidden)]
    pub struct FromFormGeneratedContext<'__f> {
        __opts: ::rocket::form::prelude::Options,
        __errors: ::rocket::form::prelude::Errors<'__f>,
        __parent: ::std::option::Option<&'__f ::rocket::form::prelude::Name>,
        a: ::std::option::Option<
            <String as ::rocket::form::prelude::FromForm<'__f>>::Context,
        >,
        b: ::std::option::Option<
            <u32 as ::rocket::form::prelude::FromForm<'__f>>::Context,
        >,
    }
    #[allow(unused_imports)]
    use ::rocket::http::uncased::AsUncased;
    impl<'__f> ::rocket::form::prelude::FromForm<'__f> for Dog {
        type Context = FromFormGeneratedContext<'__f>;
        fn init(__opts: ::rocket::form::prelude::Options) -> Self::Context {
            Self::Context {
                __opts,
                __errors: ::rocket::form::prelude::Errors::new(),
                __parent: ::std::option::Option::None,
                a: ::std::option::Option::None,
                b: ::std::option::Option::None,
            }
        }
        fn push_value(
            __c: &mut Self::Context,
            __f: ::rocket::form::prelude::ValueField<'__f>,
        ) {
            __c.__parent = __f.name.parent();
            match __f.name.key_lossy().as_str() {
                "a" => {
                    <String as ::rocket::form::prelude::FromForm<
                        '__f,
                    >>::push_value(
                        {
                            let __o = __c.__opts;
                            __c.a
                                .get_or_insert_with(|| <String as ::rocket::form::prelude::FromForm<
                                    '__f,
                                >>::init(__o))
                        },
                        __f.shift(),
                    );
                }
                "b" => {
                    <u32 as ::rocket::form::prelude::FromForm<
                        '__f,
                    >>::push_value(
                        {
                            let __o = __c.__opts;
                            __c.b
                                .get_or_insert_with(|| <u32 as ::rocket::form::prelude::FromForm<
                                    '__f,
                                >>::init(__o))
                        },
                        __f.shift(),
                    );
                }
                __k if __k == "_method" || !__c.__opts.strict => {}
                _ => __c.__errors.push(__f.unexpected()),
            }
        }
        #[allow(
            clippy::let_unit_value,
            clippy::type_complexity,
            clippy::type_repetition_in_bounds,
            clippy::used_underscore_binding
        )]
        fn push_data<'life0, 'life1, 'async_trait>(
            __c: &'life0 mut FromFormGeneratedContext<'__f>,
            __f: ::rocket::form::prelude::DataField<'__f, 'life1>,
        ) -> ::core::pin::Pin<
            Box<
                dyn ::core::future::Future<
                    Output = (),
                > + ::core::marker::Send + 'async_trait,
            >,
        >
        where
            '__f: 'async_trait,
            'life0: 'async_trait,
            'life1: 'async_trait,
        {
            Box::pin(async move {
                let __c = __c;
                let __f = __f;
                let _: () = {
                    __c.__parent = __f.name.parent();
                    match __f.name.key_lossy().as_str() {
                        "a" => {
                            let _fut = <String as ::rocket::form::prelude::FromForm<
                                '__f,
                            >>::push_data(
                                {
                                    let __o = __c.__opts;
                                    __c.a
                                        .get_or_insert_with(|| <String as ::rocket::form::prelude::FromForm<
                                            '__f,
                                        >>::init(__o))
                                },
                                __f.shift(),
                            );
                            _fut.await;
                        }
                        "b" => {
                            let _fut = <u32 as ::rocket::form::prelude::FromForm<
                                '__f,
                            >>::push_data(
                                {
                                    let __o = __c.__opts;
                                    __c.b
                                        .get_or_insert_with(|| <u32 as ::rocket::form::prelude::FromForm<
                                            '__f,
                                        >>::init(__o))
                                },
                                __f.shift(),
                            );
                            _fut.await;
                        }
                        __k if __k == "_method" || !__c.__opts.strict => {}
                        _ => __c.__errors.push(__f.unexpected()),
                    }
                };
            })
        }
        fn finalize(
            mut __c: Self::Context,
        ) -> ::std::result::Result<Self, ::rocket::form::prelude::Errors<'__f>> {
            #[allow(unused_imports)]
            use ::rocket::form::prelude::validate::*;
            let a = match {
                let __name = ::rocket::form::prelude::NameBuf::from((__c.__parent, "a"));
                let __opts = __c.__opts;
                __c.a
                    .map_or_else(
                        || {
                            {
                                <String as ::rocket::form::prelude::FromForm<
                                    '__f,
                                >>::default(__opts)
                            }
                                .ok_or_else(|| {
                                    ::rocket::form::prelude::ErrorKind::Missing.into()
                                })
                        },
                        <String as ::rocket::form::prelude::FromForm<'__f>>::finalize,
                    )
                    .and_then(|a| {
                        let mut __es = ::rocket::form::prelude::Errors::new();
                        __es.is_empty().then(|| a).ok_or(__es)
                    })
                    .map_err(|__e| __e.with_name(__name))
                    .map_err(|__e| {
                        __e
                            .is_empty()
                            .then(|| ::rocket::form::prelude::ErrorKind::Unknown.into())
                            .unwrap_or(__e)
                    })
            } {
                ::std::result::Result::Ok(a) => ::std::option::Option::Some(a),
                ::std::result::Result::Err(__e) => {
                    __c.__errors.extend(__e);
                    ::std::option::Option::None
                }
            };
            let b = match {
                let __name = ::rocket::form::prelude::NameBuf::from((__c.__parent, "b"));
                let __opts = __c.__opts;
                __c.b
                    .map_or_else(
                        || {
                            {
                                <u32 as ::rocket::form::prelude::FromForm<
                                    '__f,
                                >>::default(__opts)
                            }
                                .ok_or_else(|| {
                                    ::rocket::form::prelude::ErrorKind::Missing.into()
                                })
                        },
                        <u32 as ::rocket::form::prelude::FromForm<'__f>>::finalize,
                    )
                    .and_then(|b| {
                        let mut __es = ::rocket::form::prelude::Errors::new();
                        __es.is_empty().then(|| b).ok_or(__es)
                    })
                    .map_err(|__e| __e.with_name(__name))
                    .map_err(|__e| {
                        __e
                            .is_empty()
                            .then(|| ::rocket::form::prelude::ErrorKind::Unknown.into())
                            .unwrap_or(__e)
                    })
            } {
                ::std::result::Result::Ok(b) => ::std::option::Option::Some(b),
                ::std::result::Result::Err(__e) => {
                    __c.__errors.extend(__e);
                    ::std::option::Option::None
                }
            };
            if !__c.__errors.is_empty() {
                return ::std::result::Result::Err(__c.__errors);
            }
            let __o = Self {
                a: a.unwrap(),
                b: b.unwrap(),
            };
            if !__c.__errors.is_empty() {
                return ::std::result::Result::Err(__c.__errors);
            }
            Ok(__o)
        }
    }
};
pub fn route(num: u32, a: String, dog: Dog) {}
#[doc(hidden)]
#[allow(non_camel_case_types)]
/// Rocket code generated proxy structure.
pub struct route {}
/// Rocket code generated proxy static conversion implementations.
impl route {
    #[allow(non_snake_case, unreachable_patterns, unreachable_code)]
    fn into_info(self) -> ::rocket::route::StaticInfo {
        fn monomorphized_function<'__r>(
            __req: &'__r ::rocket::request::Request<'_>,
            __data: ::rocket::data::Data<'__r>,
        ) -> ::rocket::route::BoxFuture<'__r> {
            ::std::boxed::Box::pin(async move {
                let __rocket_num: u32 = match __req.routed_segment(1usize) {
                    ::std::option::Option::Some(__s) => {
                        match <u32 as ::rocket::request::FromParam>::from_param(__s) {
                            ::std::result::Result::Ok(__v) => __v,
                            ::std::result::Result::Err(__error) => {
                                return {
                                    {
                                        let lvl = ::log::Level::Warn;
                                        if lvl <= ::log::STATIC_MAX_LEVEL
                                            && lvl <= ::log::max_level()
                                        {
                                            ::log::__private_api_log(
                                                format_args!(
                                                    "`{0}: {1}` param guard parsed forwarding with error {2:?}",
                                                    "num",
                                                    "u32",
                                                    __error,
                                                ),
                                                lvl,
                                                &("_", "example", "alohomora/example/example.rs", 9u32),
                                            );
                                        }
                                    };
                                    ::rocket::outcome::Outcome::Forward(__data)
                                };
                            }
                        }
                    }
                    ::std::option::Option::None => {
                        {
                            let lvl = ::log::Level::Error;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api_log(
                                    format_args!(
                                        "Internal invariant broken: dyn param not found.",
                                    ),
                                    lvl,
                                    &("_", "example", "alohomora/example/example.rs", 10u32),
                                );
                            }
                        };
                        {
                            let lvl = ::log::Level::Error;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api_log(
                                    format_args!(
                                        "Please report this to the Rocket issue tracker.",
                                    ),
                                    lvl,
                                    &("_", "example", "alohomora/example/example.rs", 10u32),
                                );
                            }
                        };
                        {
                            let lvl = ::log::Level::Error;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api_log(
                                    format_args!(
                                        "https://github.com/SergioBenitez/Rocket/issues",
                                    ),
                                    lvl,
                                    &("_", "example", "alohomora/example/example.rs", 10u32),
                                );
                            }
                        };
                        return ::rocket::outcome::Outcome::Forward(__data);
                    }
                };
                let mut __e = ::rocket::form::prelude::Errors::new();
                let mut __rocket_dog = <Dog as ::rocket::form::FromForm>::init(
                    ::rocket::form::prelude::Options::Lenient,
                );
                let mut __rocket_a = <String as ::rocket::form::FromForm>::init(
                    ::rocket::form::prelude::Options::Lenient,
                );
                for _f in __req.query_fields() {
                    let _raw = (_f.name.source().as_str(), _f.value);
                    let _key = _f.name.key_lossy().as_str();
                    match (_raw, _key) {
                        (_, "dog") => {
                            <Dog as ::rocket::form::FromForm>::push_value(
                                &mut __rocket_dog,
                                _f.shift(),
                            )
                        }
                        (_, "a") => {
                            <String as ::rocket::form::FromForm>::push_value(
                                &mut __rocket_a,
                                _f.shift(),
                            )
                        }
                        _ => {}
                    }
                }
                let __rocket_dog = match <Dog as ::rocket::form::FromForm>::finalize(
                    __rocket_dog,
                ) {
                    ::std::result::Result::Ok(_v) => ::std::option::Option::Some(_v),
                    ::std::result::Result::Err(_err) => {
                        __e.extend(
                            _err.with_name(::rocket::form::prelude::NameView::new("dog")),
                        );
                        ::std::option::Option::None
                    }
                };
                let __rocket_a = match <String as ::rocket::form::FromForm>::finalize(
                    __rocket_a,
                ) {
                    ::std::result::Result::Ok(_v) => ::std::option::Option::Some(_v),
                    ::std::result::Result::Err(_err) => {
                        __e.extend(
                            _err.with_name(::rocket::form::prelude::NameView::new("a")),
                        );
                        ::std::option::Option::None
                    }
                };
                if !__e.is_empty() {
                    {
                        let lvl = ::log::Level::Warn;
                        if lvl <= ::log::STATIC_MAX_LEVEL && lvl <= ::log::max_level() {
                            ::log::__private_api_log(
                                format_args!("query string failed to match declared route"),
                                lvl,
                                &("_", "example", "alohomora/example/example.rs", 9u32),
                            );
                        }
                    };
                    for _err in __e {
                        {
                            let lvl = ::log::Level::Warn;
                            if lvl <= ::log::STATIC_MAX_LEVEL
                                && lvl <= ::log::max_level()
                            {
                                ::log::__private_api_log(
                                    format_args!("{0}", _err),
                                    lvl,
                                    &("_", "example", "alohomora/example/example.rs", 9u32),
                                );
                            }
                        };
                    }
                    return ::rocket::outcome::Outcome::Forward(__data);
                }
                let __rocket_dog = __rocket_dog.unwrap();
                let __rocket_a = __rocket_a.unwrap();
                let ___responder = route(__rocket_num, __rocket_a, __rocket_dog);
                ::rocket::route::Outcome::from(__req, ___responder)
            })
        }
        ::rocket::route::StaticInfo {
            name: "route",
            method: ::rocket::http::Method::Get,
            uri: "/route/<num>?<dog>&<a>",
            handler: monomorphized_function,
            format: ::std::option::Option::None,
            rank: ::std::option::Option::None,
            sentinels: <[_]>::into_vec(
                #[rustc_box]
                ::alloc::boxed::Box::new([
                    {
                        #[allow(unused_imports)]
                        use ::rocket::sentinel::resolution::{
                            Resolve, DefaultSentinel as _,
                        };
                        ::rocket::sentinel::Sentry {
                            type_id: std::any::TypeId::of::<u32>(),
                            type_name: std::any::type_name::<u32>(),
                            parent: None,
                            location: ("alohomora/example/example.rs", 10u32, 19u32),
                            specialized: Resolve::<u32>::SPECIALIZED,
                            abort: Resolve::<u32>::abort,
                        }
                    },
                    {
                        #[allow(unused_imports)]
                        use ::rocket::sentinel::resolution::{
                            Resolve, DefaultSentinel as _,
                        };
                        ::rocket::sentinel::Sentry {
                            type_id: std::any::TypeId::of::<Dog>(),
                            type_name: std::any::type_name::<Dog>(),
                            parent: None,
                            location: ("alohomora/example/example.rs", 10u32, 40u32),
                            specialized: Resolve::<Dog>::SPECIALIZED,
                            abort: Resolve::<Dog>::abort,
                        }
                    },
                    {
                        #[allow(unused_imports)]
                        use ::rocket::sentinel::resolution::{
                            Resolve, DefaultSentinel as _,
                        };
                        ::rocket::sentinel::Sentry {
                            type_id: std::any::TypeId::of::<String>(),
                            type_name: std::any::type_name::<String>(),
                            parent: None,
                            location: ("alohomora/example/example.rs", 10u32, 27u32),
                            specialized: Resolve::<String>::SPECIALIZED,
                            abort: Resolve::<String>::abort,
                        }
                    },
                ]),
            ),
        }
    }
    #[doc(hidden)]
    pub fn into_route(self) -> ::rocket::Route {
        self.into_info().into()
    }
}
#[doc(hidden)]
pub use rocket_uri_macro_route_8790436059816431531 as rocket_uri_macro_route;
fn main() {
    rocket::build()
        .mount(
            "/",
            {
                let ___vec: ::std::vec::Vec<::rocket::Route> = <[_]>::into_vec(
                    #[rustc_box]
                    ::alloc::boxed::Box::new([
                        {
                            let ___struct = route {};
                            let ___item: ::rocket::Route = ___struct.into_route();
                            ___item
                        },
                    ]),
                );
                ___vec
            },
        );
}
