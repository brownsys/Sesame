extern crate futures;

use futures::future::BoxFuture;
use std::fmt::Debug;
use std::net::{IpAddr, SocketAddr};
use std::option::Option;
use std::result::Result;

use crate::bbox::BBox;
use crate::rocket::cookie::BBoxCookieJar;
use crate::rocket::data::BBoxData;
use crate::rocket::form::{BBoxDataField, BBoxFormResult, BBoxValueField, FromBBoxForm};

pub type BBoxRequestOutcome<T, E> = rocket::outcome::Outcome<T, (rocket::http::Status, E), ()>;

// Request
pub struct BBoxRequest<'a, 'r> {
    request: &'a rocket::Request<'r>,
    jar: BBoxCookieJar<'a>,
}

impl<'a, 'r> BBoxRequest<'a, 'r> {
    pub fn new(request: &'a rocket::Request<'r>) -> Self {
        BBoxRequest {
            request: request,
            jar: BBoxCookieJar::new(request.cookies()),
        }
    }

    pub(crate) fn get_request(&self) -> &'a rocket::Request<'r> {
        self.request
    }

    pub fn client_ip(&self) -> Option<BBox<IpAddr>> {
        match self.request.client_ip() {
            Option::None => Option::None,
            Option::Some(ip) => Option::Some(BBox::new(ip, vec![])),
        }
    }
    pub fn remote(&self) -> Option<BBox<SocketAddr>> {
        match self.request.remote() {
            Option::None => Option::None,
            Option::Some(sock) => Option::Some(BBox::new(sock, vec![])),
        }
    }

    pub fn cookies(&self) -> &BBoxCookieJar {
        &self.jar
    }

    #[inline]
    pub fn query_fields(&self) -> impl Iterator<Item = BBoxValueField<'_>> {
        self.request.query_fields().map(|field| BBoxValueField {
            name: field.name,
            value: BBox::new(field.value.to_string(), vec![]),
            plain_value: field.value,
        })
    }

    // Use this to retrieve (boxed) guards, e.g. ApiKey struct with BBoxes inside.
    pub fn guard<'z, 'x, T>(&'x self) -> BoxFuture<'z, BBoxRequestOutcome<T, T::BBoxError>>
    where
        T: FromBBoxRequest<'x> + 'z,
        'x: 'z,
        'r: 'z,
    {
        T::from_bbox_request(self)
    }

    // Use this to retrieve (boxed) parameters in the url (e.g. /endpoint/<id>).
    pub fn param<'x, T>(&'x self, n: usize) -> Option<Result<T, T::BBoxError>>
    where
        T: FromBBoxParam,
    {
        let res = self.request.param::<String>(n)?;
        let res = res.unwrap();
        Some(T::from_bbox_param(BBox::new(res, vec![])))
    }

    // Retrieve (boxed) get parameter(s) that has given name (e.g. /endpoint/<id>?a=<THIS>)
    pub fn query_value<'x, T>(&'x self, name: &str) -> Option<BBoxFormResult<'x, T>>
    where
        T: FromBBoxForm<'x>,
    {
        match self.request.query_value::<FromFormWrapper<T>>(name) {
            Option::None => Option::None,
            Option::Some(result) => Option::Some(match result {
                Result::Ok(converter) => Result::Ok(converter.0),
                Result::Err(e) => Result::Err(e),
            }),
        }
    }
}

// Our own FromParam trait, applications likely never need to use this themselves.
pub trait FromBBoxParam: Sized {
    type BBoxError: Debug;
    fn from_bbox_param(param: BBox<String>) -> Result<Self, Self::BBoxError>;
}

// Our own FromRequest trait, receives an instance of our Request struct.
// This is used for guards.
#[rocket::async_trait]
pub trait FromBBoxRequest<'r>: Sized {
    type BBoxError: Debug;
    async fn from_bbox_request(
        request: &'r BBoxRequest<'r, '_>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError>;
}

// Private struct used in our code to make FromBBoxForm compatible with
// rocket's FromForm.
pub(super) struct FromFormWrapper<T>(pub(super) T);

#[rocket::async_trait]
impl<'r, T: FromBBoxForm<'r>> rocket::form::FromForm<'r> for FromFormWrapper<T> {
    type Context = T::BBoxContext;

    fn init(opts: rocket::form::Options) -> Self::Context {
        T::bbox_init(opts)
    }

    fn push_value(ctxt: &mut Self::Context, field: rocket::form::ValueField<'r>) {
        let bbox_field = BBoxValueField {
            name: field.name,
            value: BBox::new(field.value.to_string(), vec![]),
            plain_value: field.value,
        };
        T::bbox_push_value(ctxt, bbox_field);
    }

    async fn push_data<'life0, 'life1>(
        ctxt: &'life0 mut Self::Context,
        field: rocket::form::DataField<'r, 'life1>,
    ) {
        let bbox_field = BBoxDataField {
            name: field.name,
            content_type: field.content_type,
            request: BBoxRequest::new(field.request),
            data: BBoxData::new(field.data),
        };
        T::bbox_push_data(ctxt, bbox_field).await
    }

    fn finalize(ctxt: Self::Context) -> rocket::form::Result<'r, Self> {
        match T::bbox_finalize(ctxt) {
            Result::Err(e) => Result::Err(e),
            Result::Ok(data) => Result::Ok(FromFormWrapper(data)),
        }
    }

    // Provided methods
    fn push_error(ctxt: &mut Self::Context, error: rocket::form::Error<'r>) {
        T::bbox_push_error(ctxt, error);
    }
    fn default(opts: rocket::form::Options) -> Option<Self> {
        match T::bbox_default(opts) {
            Option::None => Option::None,
            Option::Some(data) => Option::Some(FromFormWrapper(data)),
        }
    }
}

impl FromBBoxParam for BBox<String> {
    type BBoxError = ();

    #[inline(always)]
    fn from_bbox_param(param: BBox<String>) -> Result<Self, Self::BBoxError> {
        Result::Ok(param)
    }
}

// Implement FromBBoxParam for standard types.
macro_rules! impl_param_via_fromstr {
  ($($T:ident),+ $(,)?) => ($(
      impl FromBBoxParam for BBox<$T> {
        type BBoxError = String;

        #[inline(always)]
        fn from_bbox_param(param: BBox<String>) -> Result<Self, Self::BBoxError> {
          use std::str::FromStr;
          match <$T as FromStr>::from_str(&param.t) {
            Result::Err(_) => Result::Err(String::from("Cannot parse <boxed> param")),
            Result::Ok(parsed) => Result::Ok(param.map(|_| parsed)),
          }
        }
      }
  )+)
}

use std::net::{Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6};
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};
impl_param_via_fromstr!(
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    f32,
    f64,
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
    NonZeroIsize,
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroUsize,
    bool,
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
    SocketAddrV4,
    SocketAddrV6,
    SocketAddr,
);

// Implement FromBBoxParam for a few other types that rocket controls safetly
// outside application reach.
use std::path::PathBuf;
impl FromBBoxParam for BBox<PathBuf> {
    type BBoxError = String;
    #[inline(always)]
    fn from_bbox_param(param: BBox<String>) -> Result<Self, Self::BBoxError> {
        match <PathBuf as rocket::request::FromParam>::from_param(&param.t) {
            Result::Err(_) => Result::Err(String::from("Cannot parse <boxed> param")),
            Result::Ok(parsed) => Result::Ok(param.map(|_| parsed)),
        }
    }
}
impl<T: FromBBoxParam> FromBBoxParam for Option<T> {
    type BBoxError = ();
    #[inline(always)]
    fn from_bbox_param(param: BBox<String>) -> Result<Self, Self::BBoxError> {
        match T::from_bbox_param(param) {
            Result::Ok(parsed) => Result::Ok(Option::Some(parsed)),
            Result::Err(_) => Result::Ok(Option::None),
        }
    }
}

// Implement FromBBoxRequest for some standard types.
#[rocket::async_trait]
impl<'r> FromBBoxRequest<'r> for BBox<IpAddr> {
    type BBoxError = std::convert::Infallible;
    async fn from_bbox_request(
        request: &'r BBoxRequest<'r, '_>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        match request.client_ip() {
            Option::Some(addr) => BBoxRequestOutcome::Success(addr),
            Option::None => BBoxRequestOutcome::Forward(()),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromBBoxRequest<'r> for BBox<SocketAddr> {
    type BBoxError = std::convert::Infallible;
    async fn from_bbox_request(
        request: &'r BBoxRequest<'r, '_>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        match request.remote() {
            Option::Some(addr) => BBoxRequestOutcome::Success(addr),
            Option::None => BBoxRequestOutcome::Forward(()),
        }
    }
}

#[rocket::async_trait]
impl<'r> FromBBoxRequest<'r> for &BBoxCookieJar<'r> {
    type BBoxError = std::convert::Infallible;
    async fn from_bbox_request(
        request: &'r BBoxRequest<'r, '_>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        BBoxRequestOutcome::Success(request.cookies())
    }
}

#[rocket::async_trait]
impl<'r> FromBBoxRequest<'r> for rocket::http::Method {
    type BBoxError = std::convert::Infallible;
    async fn from_bbox_request(
        request: &'r BBoxRequest<'r, '_>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        BBoxRequestOutcome::Success(request.get_request().method())
    }
}

#[rocket::async_trait]
impl<'r, T: Send + Sync + 'static> FromBBoxRequest<'r> for &'r rocket::State<T> {
    type BBoxError = ();
    async fn from_bbox_request(
        request: &'r BBoxRequest<'r, '_>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        <&rocket::State<T> as rocket::request::FromRequest>::from_request(request.get_request())
            .await
    }
}
