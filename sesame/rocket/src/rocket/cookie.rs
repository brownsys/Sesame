extern crate time;

use std::borrow::Cow;
use std::option::Option;
use time::{Duration, OffsetDateTime};

use sesame::context::{Context, ContextData};
use sesame::error::SesameResult;
use sesame::extensions::{ExtensionContext, SesameExtension};
use sesame::pcon::PCon;
use sesame::policy::{Policy, Reason, RefPolicy};

use crate::policy::FrontendPolicy;

// Cookies are build from PCons, should they also be built from non pcons?
pub struct PConCookieBuilder<'c, P: FrontendPolicy> {
    pub(self) builder: cookie::CookieBuilder<'c>,
    pub(self) value: PCon<Cow<'c, str>, P>,
}
impl<'c, P: FrontendPolicy> PConCookieBuilder<'c, P> {
    pub fn expires(self, when: OffsetDateTime) -> Self {
        Self {
            builder: self.builder.expires(when),
            value: self.value,
        }
    }
    pub fn max_age(self, value: Duration) -> Self {
        Self {
            builder: self.builder.max_age(value),
            value: self.value,
        }
    }
    pub fn domain<D: Into<Cow<'c, str>>>(self, value: D) -> Self {
        Self {
            builder: self.builder.domain(value),
            value: self.value,
        }
    }
    pub fn path<X: Into<Cow<'c, str>>>(self, path: X) -> Self {
        Self {
            builder: self.builder.path(path),
            value: self.value,
        }
    }
    pub fn secure(self, value: bool) -> Self {
        Self {
            builder: self.builder.secure(value),
            value: self.value,
        }
    }
    pub fn http_only(self, value: bool) -> Self {
        Self {
            builder: self.builder.http_only(value),
            value: self.value,
        }
    }
    pub fn same_site(self, value: rocket::http::SameSite) -> Self {
        Self {
            builder: self.builder.same_site(value),
            value: self.value,
        }
    }
    pub fn permanent(self) -> Self {
        Self {
            builder: self.builder.permanent(),
            value: self.value,
        }
    }
    pub fn finish(self) -> PConCookie<'c, P> {
        PConCookie {
            cookie: PConCookieEnum::Write(self.builder.finish(), self.value),
        }
    }
}

// Cookies are pcon-ed by default.
enum PConCookieEnum<'a, P: FrontendPolicy> {
    Read(&'a rocket::http::Cookie<'static>, P),
    Write(rocket::http::Cookie<'a>, PCon<Cow<'a, str>, P>),
}
pub struct PConCookie<'a, P: FrontendPolicy> {
    pub(self) cookie: PConCookieEnum<'a, P>,
}

impl<'c, P: FrontendPolicy> PConCookie<'c, P> {
    pub fn new<N: Into<Cow<'c, str>>, V: Into<Cow<'c, str>>>(
        name: N,
        value: PCon<V, P>,
    ) -> PConCookie<'c, P> {
        Self::build(name, value).finish()
    }

    pub fn build<N: Into<Cow<'c, str>>, V: Into<Cow<'c, str>>>(
        name: N,
        value: PCon<V, P>,
    ) -> PConCookieBuilder<'c, P> {
        PConCookieBuilder {
            builder: rocket::http::Cookie::build(name, ""),
            value: value.into_pcon(),
        }
    }

    pub fn name(&self) -> &str {
        match &self.cookie {
            PConCookieEnum::Read(cookie, _) => cookie.name(),
            PConCookieEnum::Write(cookie, _) => cookie.name(),
        }
    }

    pub fn value(&self) -> PCon<&str, RefPolicy<P>> {
        match &self.cookie {
            PConCookieEnum::Read(cookie, policy) => {
                PCon::new(cookie.value(), RefPolicy::new(policy))
            }
            PConCookieEnum::Write(_, pcon) => pcon.as_ref_pcon(),
        }
    }
}

impl<'c, P: FrontendPolicy> From<PConCookie<'c, P>> for PCon<String, P> {
    fn from(cookie: PConCookie<'c, P>) -> PCon<String, P> {
        match cookie.cookie {
            PConCookieEnum::Read(cookie, policy) => PCon::new(String::from(cookie.value()), policy),
            PConCookieEnum::Write(_name, pcon) => pcon.into_pcon(),
        }
    }
}

// Extension for checking cookie policy and adding it to jar.
struct CookieExtension<'a, 'r> {
    jar: &'a rocket::http::CookieJar<'r>,
    cookie: &'a rocket::http::Cookie<'static>,
}
impl<'a, 'r> CookieExtension<'a, 'r> {
    pub fn new(
        jar: &'a rocket::http::CookieJar<'r>,
        cookie: &'a rocket::http::Cookie<'static>,
    ) -> Self {
        Self { jar, cookie }
    }
}
impl<'a, 'r, P: Policy> SesameExtension<Cow<'static, str>, P, ()> for CookieExtension<'a, 'r> {
    fn apply(&mut self, data: Cow<'static, str>, _policy: P) -> () {
        let mut cookie = self.cookie.clone();
        cookie.set_value(data);
        self.jar.add(cookie)
    }
}

// Cookie jar gives and takes cookies that are pcon-ed.
#[derive(Clone)]
pub struct PConCookieJar<'a, 'r> {
    jar: &'a rocket::http::CookieJar<'r>,
    request: &'a rocket::Request<'r>,
}
impl<'a, 'r> PConCookieJar<'a, 'r> {
    pub(crate) fn new(
        jar: &'a rocket::http::CookieJar<'r>,
        request: &'a rocket::Request<'r>,
    ) -> Self {
        PConCookieJar { jar, request }
    }

    pub fn add<P: FrontendPolicy, D: ContextData>(
        &self,
        cookie: PConCookie<'static, P>,
        ctx: Context<D>,
    ) -> SesameResult<()> {
        match cookie.cookie {
            PConCookieEnum::Read(_, _) => {
                unreachable!("Create the cookie yourself then add it.");
            }
            PConCookieEnum::Write(cookie, pcon) => {
                let ctx = ExtensionContext::new(ctx);
                let reason = Reason::Cookie(cookie.name());
                let mut ext = CookieExtension::new(self.jar, &cookie);
                pcon.checked_extension(&mut ext, &ctx, reason)
            }
        }
    }
    pub fn get<P: FrontendPolicy>(&self, name: &str) -> Option<PConCookie<'a, P>> {
        match self.jar.get(name) {
            None => None,
            Some(cookie) => {
                let p = P::from_cookie(name, cookie, self.request);
                Some(PConCookie {
                    cookie: PConCookieEnum::Read(cookie, p),
                })
            }
        }
    }
    pub fn remove<P: FrontendPolicy>(&self, cookie: PConCookie<'static, P>) {
        match cookie.cookie {
            PConCookieEnum::Read(cookie, _) => self.jar.remove(cookie.clone()),
            PConCookieEnum::Write(_, _) => {
                unreachable!("Get the cookie using get then remove it")
            }
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.jar.iter().map(|cookie| cookie.name())
    }
}
