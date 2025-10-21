extern crate time;

use std::borrow::Cow;
use std::option::Option;
use time::{Duration, OffsetDateTime};

use sesame::bbox::BBox;
use sesame::context::{Context, ContextData};
use sesame::error::SesameResult;
use sesame::extensions::{ExtensionContext, SesameExtension};
use sesame::policy::{Policy, Reason, RefPolicy};

use crate::policy::FrontendPolicy;

// Cookies are build from BBoxes, should they also be built from non bboxes?
pub struct BBoxCookieBuilder<'c, P: FrontendPolicy> {
    pub(self) builder: cookie::CookieBuilder<'c>,
    pub(self) value: BBox<Cow<'c, str>, P>,
}
impl<'c, P: FrontendPolicy> BBoxCookieBuilder<'c, P> {
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
    pub fn finish(self) -> BBoxCookie<'c, P> {
        BBoxCookie {
            cookie: BBoxCookieEnum::Write(self.builder.finish(), self.value),
        }
    }
}

// Cookies are bboxed by default.
enum BBoxCookieEnum<'a, P: FrontendPolicy> {
    Read(&'a rocket::http::Cookie<'static>, P),
    Write(rocket::http::Cookie<'a>, BBox<Cow<'a, str>, P>),
}
pub struct BBoxCookie<'a, P: FrontendPolicy> {
    pub(self) cookie: BBoxCookieEnum<'a, P>,
}

impl<'c, P: FrontendPolicy> BBoxCookie<'c, P> {
    pub fn new<N: Into<Cow<'c, str>>, V: Into<Cow<'c, str>>>(
        name: N,
        value: BBox<V, P>,
    ) -> BBoxCookie<'c, P> {
        Self::build(name, value).finish()
    }

    pub fn build<N: Into<Cow<'c, str>>, V: Into<Cow<'c, str>>>(
        name: N,
        value: BBox<V, P>,
    ) -> BBoxCookieBuilder<'c, P> {
        BBoxCookieBuilder {
            builder: rocket::http::Cookie::build(name, ""),
            value: value.into_bbox(),
        }
    }

    pub fn name(&self) -> &str {
        match &self.cookie {
            BBoxCookieEnum::Read(cookie, _) => cookie.name(),
            BBoxCookieEnum::Write(cookie, _) => cookie.name(),
        }
    }

    pub fn value(&self) -> BBox<&str, RefPolicy<P>> {
        match &self.cookie {
            BBoxCookieEnum::Read(cookie, policy) => {
                BBox::new(cookie.value(), RefPolicy::new(policy))
            }
            BBoxCookieEnum::Write(_, bbox) => bbox.as_ref_bbox(),
        }
    }
}

impl<'c, P: FrontendPolicy> From<BBoxCookie<'c, P>> for BBox<String, P> {
    fn from(cookie: BBoxCookie<'c, P>) -> BBox<String, P> {
        match cookie.cookie {
            BBoxCookieEnum::Read(cookie, policy) => BBox::new(String::from(cookie.value()), policy),
            BBoxCookieEnum::Write(_name, bbox) => bbox.into_bbox(),
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

// Cookie jar gives and takes cookies that are bboxed.
#[derive(Clone)]
pub struct BBoxCookieJar<'a, 'r> {
    jar: &'a rocket::http::CookieJar<'r>,
    request: &'a rocket::Request<'r>,
}
impl<'a, 'r> BBoxCookieJar<'a, 'r> {
    pub(crate) fn new(
        jar: &'a rocket::http::CookieJar<'r>,
        request: &'a rocket::Request<'r>,
    ) -> Self {
        BBoxCookieJar { jar, request }
    }

    pub fn add<P: FrontendPolicy, D: ContextData>(
        &self,
        cookie: BBoxCookie<'static, P>,
        ctx: Context<D>,
    ) -> SesameResult<()> {
        match cookie.cookie {
            BBoxCookieEnum::Read(_, _) => {
                unreachable!("Create the cookie yourself then add it.");
            }
            BBoxCookieEnum::Write(cookie, bbox) => {
                let ctx = ExtensionContext::new(ctx);
                let reason = Reason::Cookie(cookie.name());
                let mut ext = CookieExtension::new(self.jar, &cookie);
                bbox.checked_extension(&mut ext, &ctx, reason)
            }
        }
    }
    pub fn get<P: FrontendPolicy>(&self, name: &str) -> Option<BBoxCookie<'a, P>> {
        match self.jar.get(name) {
            None => None,
            Some(cookie) => {
                let p = P::from_cookie(name, cookie, self.request);
                Some(BBoxCookie {
                    cookie: BBoxCookieEnum::Read(cookie, p),
                })
            }
        }
    }
    pub fn remove<P: FrontendPolicy>(&self, cookie: BBoxCookie<'static, P>) {
        match cookie.cookie {
            BBoxCookieEnum::Read(cookie, _) => self.jar.remove(cookie.clone()),
            BBoxCookieEnum::Write(_, _) => {
                unreachable!("Get the cookie using get then remove it")
            }
        }
    }
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.jar.iter().map(|cookie| cookie.name())
    }
}
