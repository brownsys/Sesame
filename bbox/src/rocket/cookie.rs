extern crate cookie;
extern crate time;

use std::borrow::Cow;
use std::option::Option;
use time::{Duration, OffsetDateTime};

use crate::bbox::BBox;
use crate::policy::Context;

// Cookies are build from BBoxes, should they also be built from non bboxes?
pub struct BBoxCookieBuilder<'c> {
    builder: cookie::CookieBuilder<'c>,
}
impl<'c> BBoxCookieBuilder<'c> {
    pub fn expires(self, when: OffsetDateTime) -> Self {
        Self {
            builder: self.builder.expires(when),
        }
    }
    pub fn max_age(self, value: Duration) -> Self {
        Self {
            builder: self.builder.max_age(value),
        }
    }
    pub fn domain<D: Into<Cow<'c, str>>>(self, value: D) -> Self {
        Self {
            builder: self.builder.domain(value),
        }
    }
    pub fn path<P: Into<Cow<'c, str>>>(self, path: P) -> Self {
        Self {
            builder: self.builder.path(path),
        }
    }
    pub fn secure(self, value: bool) -> Self {
        Self {
            builder: self.builder.secure(value),
        }
    }
    pub fn http_only(self, value: bool) -> Self {
        Self {
            builder: self.builder.http_only(value),
        }
    }
    pub fn same_site(self, value: cookie::SameSite) -> Self {
        Self {
            builder: self.builder.same_site(value),
        }
    }
    pub fn permanent(self) -> Self {
        Self {
            builder: self.builder.permanent(),
        }
    }
    pub fn finish(self) -> BBoxCookie<'c> {
        BBoxCookie {
            cookie: self.builder.finish(),
        }
    }
}

// Cookies are bboxed by default.
pub struct BBoxCookie<'c> {
    cookie: rocket::http::Cookie<'c>,
}
impl<'c> BBoxCookie<'c> {
    pub fn new<N: Into<Cow<'c, str>>, V: Into<Cow<'c, str>>, U: 'static, D: 'static>(
        name: N,
        value: BBox<V>,
        ctx: &Context<U, D>,
    ) -> BBoxCookie<'c> {
        let value = value.into_unbox(ctx);
        BBoxCookie {
            cookie: rocket::http::Cookie::new(name, value),
        }
    }
    pub fn build<N: Into<Cow<'c, str>>, V: Into<Cow<'c, str>>, U: 'static, D: 'static>(
        name: N,
        value: BBox<V>,
        ctx: &Context<U, D>,
    ) -> BBoxCookieBuilder<'c> {
        let value = value.into_unbox(ctx);
        BBoxCookieBuilder {
            builder: rocket::http::Cookie::build(name, value),
        }
    }

    pub fn value(&self) -> BBox<&str> {
        let value = self.cookie.value();
        // TODO(babamn): need to assign policies to cookies, similar to requests parameters perhaps?
        BBox::new(value, vec![])
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

    pub fn add(&self, cookie: BBoxCookie<'static>) {
        self.jar.add(cookie.cookie)
    }
    pub fn get(&self, name: &str) -> Option<BBoxCookie<'static>> {
        match self.jar.get(name) {
            Option::None => Option::None,
            Option::Some(cookie) => Option::Some(BBoxCookie {
                cookie: cookie.clone(),
            }),
        }
    }
    pub fn remove(&self, cookie: BBoxCookie<'static>) {
        self.jar.remove(cookie.cookie)
    }
    pub fn iter(&self) -> impl Iterator<Item = BBoxCookie<'static>> + '_ {
        self.jar.iter().map(|cookie| BBoxCookie {
            cookie: cookie.clone(),
        })
    }
}
