extern crate futures;

use futures::future::BoxFuture;
use rocket::http::ContentType;
use rocket_firebase_auth::{BearerToken, FirebaseAuth, FirebaseToken};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::net::{IpAddr, SocketAddr};
use std::option::Option;
use std::result::Result;

use crate::rocket::cookie::PConCookieJar;
use crate::rocket::form::{FromPConForm, PConFormResult, PConValueField};
use sesame::pcon::PCon;
use sesame::policy::Policy;

pub type PConRequestOutcome<T, E> = rocket::outcome::Outcome<T, (rocket::http::Status, E), ()>;

// Request
#[derive(Clone, Copy)]
pub struct PConRequest<'a, 'r> {
    request: &'a rocket::Request<'r>,
}

impl<'a, 'r> PConRequest<'a, 'r> {
    pub fn new(request: &'a rocket::Request<'r>) -> Self {
        PConRequest { request }
    }

    pub fn content_type(&self) -> Option<&ContentType> {
        self.request.content_type()
    }

    pub(crate) fn get_request(&self) -> &'a rocket::Request<'r> {
        self.request
    }

    pub fn client_ip<P: FrontendPolicy>(&self) -> Option<PCon<IpAddr, P>> {
        match self.request.client_ip() {
            None => None,
            Some(ip) => Some(PCon::new(ip, P::from_request(self.request))),
        }
    }
    pub fn remote<P: FrontendPolicy>(&self) -> Option<PCon<SocketAddr, P>> {
        match self.request.remote() {
            None => None,
            Some(sock) => Some(PCon::new(sock, P::from_request(self.request))),
        }
    }

    pub fn cookies(&self) -> PConCookieJar<'a, 'r> {
        PConCookieJar::new(self.request.cookies(), self.request)
    }

    pub fn headers(&self) -> PConHeaderMap<'a, 'r> {
        PConHeaderMap::new(self.request, self.request.headers())
    }

    pub async fn firebase_token<P: FrontendPolicy>(
        &self,
        firebase_auth: &FirebaseAuth,
    ) -> Option<PCon<FirebaseToken, P>> {
        let header = self.request.headers().get_one("Authorization")?;
        match BearerToken::try_from(header) {
            Err(_) => None,
            Ok(token) => match firebase_auth.verify(token.as_str()).await {
                Err(_) => None,
                Ok(token) => Some(PCon::new(token, P::from_request(self.request))),
            },
        }
    }

    // Use this to retrieve (boxed) guards, e.g. ApiKey struct with PCons inside.
    pub fn guard<T>(&self) -> BoxFuture<'a, PConRequestOutcome<T, T::PConError>>
    where
        T: FromPConRequest<'a, 'r> + 'a,
    {
        T::from_pcon_request(*self)
    }

    // Use this to retrieve (boxed) parameters in the url (e.g. /endpoint/<id>).
    pub fn param<T, P: FrontendPolicy>(&self, n: usize) -> Option<Result<T, T::PConError>>
    where
        T: FromPConParam<P>,
    {
        let res = self.request.param::<String>(n)?;
        let res = res.unwrap();
        Some(T::from_pcon_param(PCon::new(
            res,
            P::from_request(self.request),
        )))
    }

    // Retrieve (boxed) get parameter(s) that has given name (e.g. /endpoint/<id>?a=<THIS>)
    pub fn query_value<T>(&self, name: &str) -> Option<PConFormResult<'a, T>>
    where
        T: FromPConForm<'a, 'r>,
    {
        if self.query_fields().find(|f| f.name == name).is_none() {
            return None;
        }

        let mut ctxt = T::pcon_init(rocket::form::Options::Lenient);

        self.query_fields()
            .filter(|f| f.name == name)
            .for_each(|f| T::pcon_push_value(&mut ctxt, f.shift(), *self));

        Some(T::pcon_finalize(ctxt))
    }

    // Iterate over all query values.
    #[inline]
    pub fn query_fields(&self) -> impl Iterator<Item = PConValueField<'a>> {
        self.request.query_fields().map(|field| PConValueField {
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
pub trait FromPConRequest<'a, 'r>: Sized {
    type PConError: Debug + Send;
    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError>;
}

// Similar to FromPConRequest, but also receives the PConData (for form parsing).
// Used exclusively for Context.
#[rocket::async_trait]
pub trait FromPConRequestAndData<'a, 'r, T: Sync + FromPConData<'a, 'r>>: Sized {
    type PConError: Debug + Send;
    async fn from_pcon_request_and_data(
        request: PConRequest<'a, 'r>,
        data: &'_ T,
    ) -> PConRequestOutcome<Self, Self::PConError>;
}

// Our own FromParam trait, applications likely never need to use this themselves.
pub trait FromPConParam<P: Policy>: Sized {
    type PConError: Debug + Send;
    fn from_pcon_param(param: PCon<String, P>) -> Result<Self, Self::PConError>;
}

impl<P: Policy> FromPConParam<P> for PCon<String, P> {
    type PConError = ();

    #[inline(always)]
    fn from_pcon_param(param: PCon<String, P>) -> Result<Self, Self::PConError> {
        Ok(param)
    }
}

// Implement FromPConParam for standard types.
struct Converter {}
impl UncheckedSesameExtension for Converter {}
impl<P: Policy, R: FromStr> SesameExtension<String, P, Result<PCon<R, P>, String>> for Converter {
    fn apply(&mut self, data: String, policy: P) -> Result<PCon<R, P>, String> {
        match R::from_str(&data) {
            Err(_) => Err(String::from("Cannot parse <boxed> param")),
            Ok(parsed) => Ok(PCon::new(parsed, policy)),
        }
    }
}
impl<P: Policy> SesameExtension<String, P, Result<PCon<PathBuf, P>, &'static str>> for Converter {
    fn apply(&mut self, data: String, policy: P) -> Result<PCon<PathBuf, P>, &'static str> {
        match <PathBuf as rocket::request::FromParam>::from_param(&data) {
            Err(_) => Err("Cannot parse <boxed> param"),
            Ok(parsed) => Ok(PCon::new(parsed, policy)),
        }
    }
}

macro_rules! impl_param_via_fromstr {
    ($($T:ident),+ $(,)?) => ($(
        impl<P: Policy> FromPConParam<P> for PCon<$T, P> {
            type PConError = String;

            #[inline(always)]
            fn from_pcon_param(param: PCon<String, P>) -> Result<Self, Self::PConError> {
                param.unchecked_extension(&mut Converter {})
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

// Implement FromPConParam for a few other types that rocket controls safely
// outside application reach.
use crate::policy::FrontendPolicy;
use crate::rocket::{FromPConData, PConHeaderMap};
use sesame::extensions::{SesameExtension, UncheckedSesameExtension};
use std::path::PathBuf;
use std::str::FromStr;

impl<P: Policy> FromPConParam<P> for PCon<PathBuf, P> {
    type PConError = String;
    #[inline(always)]
    fn from_pcon_param(param: PCon<String, P>) -> Result<Self, Self::PConError> {
        let result: Result<Self, &'static str> = param.unchecked_extension(&mut Converter {});
        result.map_err(str::to_string)
    }
}
impl<P: Policy, T: FromPConParam<P>> FromPConParam<P> for Option<T> {
    type PConError = ();
    #[inline(always)]
    fn from_pcon_param(param: PCon<String, P>) -> Result<Self, Self::PConError> {
        match T::from_pcon_param(param) {
            Ok(parsed) => Ok(Some(parsed)),
            Err(_) => Ok(None),
        }
    }
}

// Implement FromPConRequest for some standard types.
#[rocket::async_trait]
impl<'a, 'r> FromPConRequest<'a, 'r> for () {
    type PConError = std::convert::Infallible;
    async fn from_pcon_request(
        _request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        PConRequestOutcome::Success(())
    }
}

/*
#[rocket::async_trait]
impl<'a, 'r, P: FrontendPolicy> FromPConRequest<'a, 'r> for PCon<IpAddr, P> {
    type PConError = std::convert::Infallible;
    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        match request.client_ip() {
            Some(addr) => PConRequestOutcome::Success(addr),
            None => PConRequestOutcome::Forward(()),
        }
    }
}

#[rocket::async_trait]
impl<'a, 'r, P: FrontendPolicy> FromPConRequest<'a, 'r> for PCon<SocketAddr, P> {
    type PConError = std::convert::Infallible;
    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        match request.remote() {
            Some(addr) => PConRequestOutcome::Success(addr),
            None => PConRequestOutcome::Forward(()),
        }
    }
}
*/

// TODO(babman): look at technical debt issue on github.
#[rocket::async_trait]
impl<'a, 'r, T: rocket::request::FromRequest<'a>, P: Policy + FrontendPolicy>
    FromPConRequest<'a, 'r> for PCon<T, P>
{
    type PConError = ();
    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        match <T as rocket::request::FromRequest>::from_request(request.get_request()).await {
            rocket::request::Outcome::Success(t) => {
                PConRequestOutcome::Success(PCon::new(t, P::from_request(request.get_request())))
            }
            rocket::request::Outcome::Forward(()) => PConRequestOutcome::Forward(()),
            rocket::request::Outcome::Failure((status, error)) => {
                println!("Error {} {:?}", status, error);
                PConRequestOutcome::Failure((status, ()))
            }
        }
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromPConRequest<'a, 'r> for PConCookieJar<'a, 'r> {
    type PConError = std::convert::Infallible;
    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        PConRequestOutcome::Success(request.cookies())
    }
}

#[rocket::async_trait]
impl<'a, 'r> FromPConRequest<'a, 'r> for rocket::http::Method {
    type PConError = std::convert::Infallible;
    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        PConRequestOutcome::Success(request.get_request().method())
    }
}

#[rocket::async_trait]
impl<'a, 'r, T: Send + Sync + 'static> FromPConRequest<'a, 'r> for &'a rocket::State<T> {
    type PConError = ();
    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        <&rocket::State<T> as rocket::request::FromRequest>::from_request(request.get_request())
            .await
    }
}
