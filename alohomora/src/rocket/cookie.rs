extern crate cookie;
extern crate time;

use std::borrow::Cow;
use std::option::Option;
use time::{Duration, OffsetDateTime};

use crate::bbox::BBox;
use crate::context::{Context, ContextData, UnprotectedContext};
use crate::policy::{FrontendPolicy, Reason, RefPolicy};

// Cookies are build from BBoxes, should they also be built from non bboxes?
pub struct BBoxCookieBuilder<'c, P: FrontendPolicy> {
    builder: cookie::CookieBuilder<'c>,
    policy: P,
}
impl<'c, P: FrontendPolicy> BBoxCookieBuilder<'c, P> {
    pub fn expires(self, when: OffsetDateTime) -> Self {
        Self {
            builder: self.builder.expires(when),
            policy: self.policy,
        }
    }
    pub fn max_age(self, value: Duration) -> Self {
        Self {
            builder: self.builder.max_age(value),
            policy: self.policy,
        }
    }
    pub fn domain<D: Into<Cow<'c, str>>>(self, value: D) -> Self {
        Self {
            builder: self.builder.domain(value),
            policy: self.policy,
        }
    }
    pub fn path<X: Into<Cow<'c, str>>>(self, path: X) -> Self {
        Self {
            builder: self.builder.path(path),
            policy: self.policy,
        }
    }
    pub fn secure(self, value: bool) -> Self {
        Self {
            builder: self.builder.secure(value),
            policy: self.policy,
        }
    }
    pub fn http_only(self, value: bool) -> Self {
        Self {
            builder: self.builder.http_only(value),
            policy: self.policy,
        }
    }
    pub fn same_site(self, value: cookie::SameSite) -> Self {
        Self {
            builder: self.builder.same_site(value),
            policy: self.policy,
        }
    }
    pub fn permanent(self) -> Self {
        Self {
            builder: self.builder.permanent(),
            policy: self.policy,
        }
    }
    pub fn finish(self) -> BBoxCookie<'c, P> {
        BBoxCookie {
            cookie: self.builder.finish(),
            policy: self.policy,
        }
    }
}

// Cookies are bboxed by default.
pub struct BBoxCookie<'c, P: FrontendPolicy> {
    cookie: rocket::http::Cookie<'c>,
    policy: P,
}

impl<'c, P: FrontendPolicy> BBoxCookie<'c, P> {
    pub fn new<N: Into<Cow<'c, str>>, V: Into<Cow<'c, str>>>(
        name: N,
        value: BBox<V, P>,
    ) -> BBoxCookie<'c, P> {
        let (t, p) = value.consume();
        BBoxCookie {
            cookie: rocket::http::Cookie::new(name, t),
            policy: p,
        }
    }

    pub fn build<N: Into<Cow<'c, str>>, V: Into<Cow<'c, str>> + Clone>(
        name: N,
        value: BBox<V, P>,
    ) -> BBoxCookieBuilder<'c, P> {
        let (t, p) = value.consume();
        BBoxCookieBuilder {
            builder: rocket::http::Cookie::build(name, t),
            policy: p,
        }
    }

    pub fn name(&self) -> &str {
        self.cookie.name()
    }

    pub fn value(&self) -> BBox<&str, RefPolicy<P>> {
        BBox::new(self.cookie.value(), RefPolicy::new(&self.policy))
    }
}

impl<'c, P: FrontendPolicy> From<BBoxCookie<'c, P>> for BBox<String, P> {
    fn from(cookie: BBoxCookie<'c, P>) -> BBox<String, P> {
        BBox::new(String::from(cookie.cookie.value()), cookie.policy)
    }
}

// Cookie jar gives and takes cookies that are bboxed.
#[derive(Clone)]
pub struct BBoxCookieJar<'a, 'r> {
    jar: &'a rocket::http::CookieJar<'r>,
    request: &'a rocket::Request<'r>,
}
impl<'a, 'r> BBoxCookieJar<'a, 'r> {
    pub fn new(jar: &'a rocket::http::CookieJar<'r>, request: &'a rocket::Request<'r>) -> Self {
        BBoxCookieJar { jar, request }
    }

    pub fn add<P: FrontendPolicy, D: ContextData>(&self, cookie: BBoxCookie<'static, P>, ctx: Context<D>) -> Result<(), ()> {
        let ctx = UnprotectedContext::from(ctx);
        if cookie.policy.check(&ctx, Reason::Cookie(cookie.name())) {
            self.jar.add(cookie.cookie);
            return Ok(());
        }
        return Err(());
    }
    pub fn get<P: FrontendPolicy>(&self, name: &str) -> Option<BBoxCookie<'static, P>> {
        match self.jar.get(name) {
            None => None,
            Some(cookie) => {
                let p = P::from_cookie(name, cookie, self.request);
                Some(BBoxCookie {
                    cookie: cookie.clone(),
                    policy: p,
                })
            },
        }
    }
    pub fn remove<P: FrontendPolicy>(&self, cookie: BBoxCookie<'static, P>) {
        self.jar.remove(cookie.cookie)
    }
    pub fn iter(&self) -> impl Iterator<Item = &str> {
        self.jar.iter().map(|cookie| cookie.name())
    }
}
