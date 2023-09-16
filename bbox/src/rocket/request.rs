extern crate futures;

use futures::future::BoxFuture;
use std::fmt::Debug;
use std::net::{IpAddr, SocketAddr};
use std::option::Option;
use std::result::Result;

use crate::bbox::BBox;
use crate::policy::{FrontendPolicy, Policy};
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
            request,
            jar: BBoxCookieJar::new(request.cookies()),
        }
    }

    pub(crate) fn get_request(&self) -> &'a rocket::Request<'r> {
        self.request
    }

    pub fn client_ip<P: FrontendPolicy>(&self) -> Option<BBox<IpAddr, P>> {
        match self.request.client_ip() {
            None => None,
            Some(ip) => Some(BBox::new(ip, P::from_request(self))),
        }
    }
    pub fn remote<P: FrontendPolicy>(&self) -> Option<BBox<SocketAddr, P>> {
        match self.request.remote() {
            None => None,
            Some(sock) => Some(BBox::new(sock, P::from_request(self))),
        }
    }

    pub fn cookies(&self) -> &BBoxCookieJar {
        &self.jar
    }

    #[inline]
    pub fn query_fields(&self) -> impl Iterator<Item = BBoxValueField<'_>> {
        self.request.query_fields().map(|field| BBoxValueField {
            name: field.name,
            value: field.value,
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
    pub fn param<'x, T, P: FrontendPolicy>(&'x self, n: usize) -> Option<Result<T, T::BBoxError>>
    where
        T: FromBBoxParam<P>,
    {
        let res = self.request.param::<String>(n)?;
        let res = res.unwrap();
        Some(T::from_bbox_param(BBox::new(res, P::from_request(self))))
    }

    // Retrieve (boxed) get parameter(s) that has given name (e.g. /endpoint/<id>?a=<THIS>)
    pub fn query_value<'x, T>(&'x self, name: &str) -> Option<BBoxFormResult<'x, T>>
    where
        T: FromBBoxForm<'x>,
    {
        match self.request.query_value::<FromFormWrapper<T>>(name) {
            None => None,
            Some(result) => Some(match result {
                Ok(converter) => Ok(converter.0),
                Err(e) => Err(e),
            }),
        }
    }

    // Returns information about the route.
    pub fn route(&self) -> Option<&'r rocket::route::Route> {
        self.request.route()
    }
}

// Our own FromParam trait, applications likely never need to use this themselves.
pub trait FromBBoxParam<P: Policy>: Sized {
    type BBoxError: Debug;
    fn from_bbox_param(param: BBox<String, P>) -> Result<Self, Self::BBoxError>;
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
impl<'r, T: FromBBoxForm<'r> + Send> rocket::form::FromForm<'r> for FromFormWrapper<T> {
    type Context = T::BBoxContext;

    fn init(opts: rocket::form::Options) -> Self::Context {
        T::bbox_init(opts)
    }

    fn push_value(ctxt: &mut Self::Context, field: rocket::form::ValueField<'r>) {
        let bbox_field = BBoxValueField {
            name: field.name,
            value: field.value,
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

    fn push_error(ctxt: &mut Self::Context, error: rocket::form::Error<'r>) {
        T::bbox_push_error(ctxt, error);
    }

    fn finalize(ctxt: Self::Context) -> rocket::form::Result<'r, Self> {
        match T::bbox_finalize(ctxt) {
            Err(e) => Err(e),
            Ok(data) => Ok(FromFormWrapper(data)),
        }
    }

    fn default(opts: rocket::form::Options) -> Option<Self> {
        match T::bbox_default(opts) {
            None => None,
            Some(data) => Some(FromFormWrapper(data)),
        }
    }
}

impl<P: Policy> FromBBoxParam<P> for BBox<String, P> {
    type BBoxError = ();

    #[inline(always)]
    fn from_bbox_param(param: BBox<String, P>) -> Result<Self, Self::BBoxError> {
        Ok(param)
    }
}

// Implement FromBBoxParam for standard types.
macro_rules! impl_param_via_fromstr {
  ($($T:ident),+ $(,)?) => ($(
      impl<P: Policy> FromBBoxParam<P> for BBox<$T, P> {
        type BBoxError = String;

        #[inline(always)]
        fn from_bbox_param(param: BBox<String, P>) -> Result<Self, Self::BBoxError> {
          use std::str::FromStr;
          match <$T as FromStr>::from_str(&param.t) {
            Err(_) => Err(String::from("Cannot parse <boxed> param")),
            Ok(parsed) => Ok(BBox::new(parsed, param.p)),
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

impl<P: Policy> FromBBoxParam<P> for BBox<PathBuf, P> {
    type BBoxError = String;
    #[inline(always)]
    fn from_bbox_param(param: BBox<String, P>) -> Result<Self, Self::BBoxError> {
        match <PathBuf as rocket::request::FromParam>::from_param(&param.t) {
            Err(_) => Err(String::from("Cannot parse <boxed> param")),
            Ok(parsed) => Ok(BBox::new(parsed, param.p)),
        }
    }
}
impl<P: Policy, T: FromBBoxParam<P>> FromBBoxParam<P> for Option<T> {
    type BBoxError = ();
    #[inline(always)]
    fn from_bbox_param(param: BBox<String, P>) -> Result<Self, Self::BBoxError> {
        match T::from_bbox_param(param) {
            Ok(parsed) => Ok(Some(parsed)),
            Err(_) => Ok(None),
        }
    }
}

// Implement FromBBoxRequest for some standard types.
#[rocket::async_trait]
impl<'r, P: FrontendPolicy> FromBBoxRequest<'r> for BBox<IpAddr, P> {
    type BBoxError = std::convert::Infallible;
    async fn from_bbox_request(
        request: &'r BBoxRequest<'r, '_>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        match request.client_ip() {
            Some(addr) => BBoxRequestOutcome::Success(addr),
            None => BBoxRequestOutcome::Forward(()),
        }
    }
}

#[rocket::async_trait]
impl<'r, P: FrontendPolicy> FromBBoxRequest<'r> for BBox<SocketAddr, P> {
    type BBoxError = std::convert::Infallible;
    async fn from_bbox_request(
        request: &'r BBoxRequest<'r, '_>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        match request.remote() {
            Some(addr) => BBoxRequestOutcome::Success(addr),
            None => BBoxRequestOutcome::Forward(()),
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
