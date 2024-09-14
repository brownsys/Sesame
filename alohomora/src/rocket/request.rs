extern crate futures;

use futures::future::BoxFuture;
use rocket::http::ContentType;
use std::fmt::Debug;
use std::net::{IpAddr, SocketAddr};
use std::option::Option;
use std::result::Result;
use rocket_firebase_auth::{BearerToken, FirebaseAuth, FirebaseToken};
use std::convert::TryFrom;

use crate::bbox::BBox;
use crate::policy::{FrontendPolicy, Policy};
use crate::rocket::cookie::BBoxCookieJar;
use crate::rocket::form::{BBoxFormResult, BBoxValueField, FromBBoxForm};

pub type BBoxRequestOutcome<T, E> = rocket::outcome::Outcome<T, (rocket::http::Status, E), ()>;

// Request
#[derive(Clone, Copy)]
pub struct BBoxRequest<'a, 'r> {
    request: &'a rocket::Request<'r>,
}

impl<'a, 'r> BBoxRequest<'a, 'r> {
    pub fn new(request: &'a rocket::Request<'r>) -> Self {
        BBoxRequest { request }
    }

    pub fn content_type(&self) -> Option<&ContentType> {
        self.request.content_type()
    }

    pub(crate) fn get_request(&self) -> &'a rocket::Request<'r> {
        self.request
    }

    pub fn client_ip<P: FrontendPolicy>(&self) -> Option<BBox<IpAddr, P>> {
        match self.request.client_ip() {
            None => None,
            Some(ip) => Some(BBox::new(ip, P::from_request(self.request))),
        }
    }
    pub fn remote<P: FrontendPolicy>(&self) -> Option<BBox<SocketAddr, P>> {
        match self.request.remote() {
            None => None,
            Some(sock) => Some(BBox::new(sock, P::from_request(self.request))),
        }
    }

    pub fn cookies(&self) -> BBoxCookieJar<'a, 'r> {
        BBoxCookieJar::new(self.request.cookies(), self.request)
    }

    pub fn headers(&self) -> BBoxHeaderMap<'a, 'r> {
        BBoxHeaderMap::new(self.request, self.request.headers())
    }

    pub async fn firebase_token<P: FrontendPolicy>(&self, firebase_auth: &FirebaseAuth)
    -> Option<BBox<FirebaseToken, P>> {
        let header = self.request.headers().get_one("Authorization")?;
        match BearerToken::try_from(header) {
            Err(_) => None,
            Ok(token) => match firebase_auth.verify(token.as_str()).await {
                Err(_) => None,
                Ok(token) => Some(BBox::new(token, P::from_request(self.request))),
            },
        }
    }

    // Use this to retrieve (boxed) guards, e.g. ApiKey struct with BBoxes inside.
    pub fn guard<T>(&self) -> BoxFuture<'a, BBoxRequestOutcome<T, T::BBoxError>>
    where
        T: FromBBoxRequest<'a, 'r> + 'a,
    {
        T::from_bbox_request(*self)
    }

    // Use this to retrieve (boxed) parameters in the url (e.g. /endpoint/<id>).
    pub fn param<T, P: FrontendPolicy>(&self, n: usize) -> Option<Result<T, T::BBoxError>>
    where
        T: FromBBoxParam<P>,
    {
        let res = self.request.param::<String>(n)?;
        let res = res.unwrap();
        Some(T::from_bbox_param(BBox::new(res, P::from_request(self.request))))
    }

    // Retrieve (boxed) get parameter(s) that has given name (e.g. /endpoint/<id>?a=<THIS>)
    pub fn query_value<T>(&self, name: &str) -> Option<BBoxFormResult<'a, T>>
    where
        T: FromBBoxForm<'a, 'r>,
    {
        if self.query_fields().find(|f| f.name == name).is_none() {
            return None;
        }

        let mut ctxt = T::bbox_init(rocket::form::Options::Lenient);

        self.query_fields()
            .filter(|f| f.name == name)
            .for_each(|f| T::bbox_push_value(&mut ctxt, f.shift(), *self));

        Some(T::bbox_finalize(ctxt))
    }

    // Iterate over all query values.
    #[inline]
    pub fn query_fields(&self) -> impl Iterator<Item = BBoxValueField<'a>> {
        self.request.query_fields().map(|field| BBoxValueField {
            name: field.name,
            value: field.value,
        })
    }

    // Returns information about the route.
    pub fn route(&self) -> Option<&'r rocket::route::Route> {
        self.request.route()
    }
}

// Our own FromRequest trait, receives an instance of our Request struct.
// This is used for guards.
#[rocket::async_trait]
pub trait FromBBoxRequest<'a, 'r>: Sized {
    type BBoxError: Debug + Send;
    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError>;
}

// Similar to FromBBoxRequest, but also receives the BBoxData (for form parsing).
// Used exclusively for Context.
#[rocket::async_trait]
pub trait FromBBoxRequestAndData<'a, 'r, T: Sync + FromBBoxData<'a, 'r>>: Sized {
    type BBoxError: Debug + Send;
    async fn from_bbox_request_and_data(
        request: BBoxRequest<'a, 'r>,
        data: &'_ T,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError>;
}

// Our own FromParam trait, applications likely never need to use this themselves.
pub trait FromBBoxParam<P: Policy>: Sized {
    type BBoxError: Debug + Send;
    fn from_bbox_param(param: BBox<String, P>) -> Result<Self, Self::BBoxError>;
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
          let (t, p) = param.consume();
          match <$T as FromStr>::from_str(&t) {
            Err(_) => Err(String::from("Cannot parse <boxed> param")),
            Ok(parsed) => Ok(BBox::new(parsed, p)),
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

// Implement FromBBoxParam for a few other types that rocket controls safely
// outside application reach.
use std::path::PathBuf;
use rocket::data::Outcome;
use crate::rocket::{BBoxData, BBoxHeaderMap, FromBBoxData};

impl<P: Policy> FromBBoxParam<P> for BBox<PathBuf, P> {
    type BBoxError = String;
    #[inline(always)]
    fn from_bbox_param(param: BBox<String, P>) -> Result<Self, Self::BBoxError> {
        let (t, p) = param.consume();
        match <PathBuf as rocket::request::FromParam>::from_param(&t) {
            Err(_) => Err(String::from("Cannot parse <boxed> param")),
            Ok(parsed) => Ok(BBox::new(parsed, p)),
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
impl<'a, 'r> FromBBoxRequest<'a, 'r> for () {
    type BBoxError = std::convert::Infallible;
    async fn from_bbox_request(
        _request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        BBoxRequestOutcome::Success(())
    }
}

/*
#[rocket::async_trait]
impl<'a, 'r, P: FrontendPolicy> FromBBoxRequest<'a, 'r> for BBox<IpAddr, P> {
    type BBoxError = std::convert::Infallible;
    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        match request.client_ip() {
            Some(addr) => BBoxRequestOutcome::Success(addr),
            None => BBoxRequestOutcome::Forward(()),
        }
    }
}

#[rocket::async_trait]
impl<'a, 'r, P: FrontendPolicy> FromBBoxRequest<'a, 'r> for BBox<SocketAddr, P> {
    type BBoxError = std::convert::Infallible;
    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        match request.remote() {
            Some(addr) => BBoxRequestOutcome::Success(addr),
            None => BBoxRequestOutcome::Forward(()),
        }
    }
}
*/

// TODO(babman): look at technical debt issue on github.
#[rocket::async_trait]
impl<'a, 'r, T: rocket::request::FromRequest<'a>, P: Policy + FrontendPolicy> FromBBoxRequest<'a, 'r> for BBox<T, P> {
    type BBoxError = ();
    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        match <T as rocket::request::FromRequest>::from_request(request.get_request()).await {
            rocket::request::Outcome::Success(t) => BBoxRequestOutcome::Success(BBox::new(t, P::from_request(request.get_request()))),
            rocket::request::Outcome::Forward(()) => BBoxRequestOutcome::Forward(()),
            rocket::request::Outcome::Failure((status, error)) => {
                println!("Error {} {:?}", status, error);
                BBoxRequestOutcome::Failure((status, ()))
            },
        }
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for BBoxCookieJar<'a, 'r> {
    type BBoxError = std::convert::Infallible;
    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        BBoxRequestOutcome::Success(request.cookies())
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for rocket::http::Method {
    type BBoxError = std::convert::Infallible;
    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        BBoxRequestOutcome::Success(request.get_request().method())
    }
}

#[rocket::async_trait]
impl<'a, 'r, T: Send + Sync + 'static> FromBBoxRequest<'a, 'r> for &'a rocket::State<T> {
    type BBoxError = ();
    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        <&rocket::State<T> as rocket::request::FromRequest>::from_request(request.get_request())
            .await
    }
}
