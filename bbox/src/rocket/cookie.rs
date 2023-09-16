extern crate cookie;
extern crate time;

use std::borrow::Cow;
use std::marker::PhantomData;
use std::option::Option;
use time::{Duration, OffsetDateTime};

use crate::bbox::BBox;
use crate::context::Context;
use crate::policy::FrontendPolicy;

// Cookies are build from BBoxes, should they also be built from non bboxes?
pub struct BBoxCookieBuilder<'c, P: FrontendPolicy> {
    builder: cookie::CookieBuilder<'c>,
    _policy: PhantomData<P>,
}
impl<'c, P: FrontendPolicy> BBoxCookieBuilder<'c, P> {
    pub fn expires(self, when: OffsetDateTime) -> Self {
        Self {
            builder: self.builder.expires(when),
            _policy: self._policy,
        }
    }
    pub fn max_age(self, value: Duration) -> Self {
        Self {
            builder: self.builder.max_age(value),
            _policy: self._policy,
        }
    }
    pub fn domain<D: Into<Cow<'c, str>>>(self, value: D) -> Self {
        Self {
            builder: self.builder.domain(value),
            _policy: self._policy,
        }
    }
    pub fn path<X: Into<Cow<'c, str>>>(self, path: X) -> Self {
        Self {
            builder: self.builder.path(path),
            _policy: self._policy,
        }
    }
    pub fn secure(self, value: bool) -> Self {
        Self {
            builder: self.builder.secure(value),
            _policy: self._policy,
        }
    }
    pub fn http_only(self, value: bool) -> Self {
        Self {
            builder: self.builder.http_only(value),
            _policy: self._policy,
        }
    }
    pub fn same_site(self, value: cookie::SameSite) -> Self {
        Self {
            builder: self.builder.same_site(value),
            _policy: self._policy,
        }
    }
    pub fn permanent(self) -> Self {
        Self {
            builder: self.builder.permanent(),
            _policy: self._policy,
        }
    }
    pub fn finish(self) -> BBoxCookie<'c, P> {
        BBoxCookie {
            cookie: self.builder.finish(),
            _policy: self._policy,
        }
    }
}

// Cookies are bboxed by default.
pub struct BBoxCookie<'c, P: FrontendPolicy> {
    cookie: rocket::http::Cookie<'c>,
    _policy: PhantomData<P>,
}
impl<'c, P: FrontendPolicy> BBoxCookie<'c, P> {
    pub fn new<N: Into<Cow<'c, str>>, V: Into<Cow<'c, str>>, U: 'static, D: 'static>(
        name: N,
        value: BBox<V, P>,
        ctx: &Context<U, D>,
    ) -> BBoxCookie<'c, P> {
        let value = value.into_unbox(ctx);
        BBoxCookie {
            cookie: rocket::http::Cookie::new(name, value),
            _policy: PhantomData,
        }
    }
    pub fn build<N: Into<Cow<'c, str>>, V: Into<Cow<'c, str>>, U: 'static, D: 'static>(
        name: N,
        value: BBox<V, P>,
        ctx: &Context<U, D>,
    ) -> BBoxCookieBuilder<'c, P> {
        let value = value.into_unbox(ctx);
        BBoxCookieBuilder {
            builder: rocket::http::Cookie::build(name, value),
            _policy: PhantomData,
        }
    }

    pub fn value(&self) -> BBox<&str, P> {
        let value = self.cookie.value();
        BBox::new(value, P::from_cookie())
    }
}

// Cookie jar gives and takes cookies that are bboxed.
pub struct BBoxCookieJar<'r> {
    jar: &'r rocket::http::CookieJar<'r>,
}
impl<'r> BBoxCookieJar<'r> {
    pub fn new(jar: &'r rocket::http::CookieJar<'r>) -> Self {
        BBoxCookieJar { jar }
    }

    pub fn add<P: FrontendPolicy>(&self, cookie: BBoxCookie<'static, P>) {
        self.jar.add(cookie.cookie)
    }
    pub fn get<P: FrontendPolicy>(&self, name: &str) -> Option<BBoxCookie<'static, P>> {
        match self.jar.get(name) {
            None => None,
            Some(cookie) => Some(BBoxCookie {
                cookie: cookie.clone(),
                _policy: PhantomData,
            }),
        }
    }
    pub fn remove<P: FrontendPolicy>(&self, cookie: BBoxCookie<'static, P>) {
        self.jar.remove(cookie.cookie)
    }
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.jar.iter().map(|cookie| cookie.name())
    }
}
