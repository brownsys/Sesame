use std::convert::TryInto;
use crate::rocket::response::{BBoxResponseResult, BBoxResponder, BBoxResponse};
use crate::rocket::request::BBoxRequest;
use crate::rocket::redirect_parameters::RedirectParams;

use dynfmt::{Format, SimpleCurlyFormat};
use rocket::http::uri::Reference;
use crate::context::{Context, ContextData};
use crate::rocket::IntoRedirectParams;

// A redirect response.
pub struct BBoxRedirect {
    redirect: rocket::response::Redirect,
}
impl BBoxRedirect {
    pub fn to<'a, P: IntoRedirectParams, D: ContextData>(url: &str, params: P, context: Context<D>) -> Self {
        let params: RedirectParams = params.into(url, context);
        let formatted_str = SimpleCurlyFormat.format(url, params.parameters).unwrap();
        BBoxRedirect {
            redirect: rocket::response::Redirect::to(Into::<String>::into(formatted_str)),
        }
    }
    pub fn to2<U: TryInto<Reference<'static>>>(url: U) -> Self {
        BBoxRedirect {
            redirect: rocket::response::Redirect::to(url)
        }
    }
}
impl<'a, 'r> BBoxResponder<'a, 'r, 'static> for BBoxRedirect {
    fn respond_to(self, request: BBoxRequest<'a, 'r>) -> BBoxResponseResult<'static> {
        match rocket::response::Responder::respond_to(self.redirect,request.get_request()) {
            Ok(response) => Ok(BBoxResponse::new(response)),
            Err(e) => Err(e),
        }
    }
}

// Unit tests.
#[cfg(test)]
mod tests {
    use crate::bbox::BBox;
    use crate::context::Context;
    use crate::policy::NoPolicy;
    use crate::rocket::BBoxRedirect;

    #[test]
    fn test_mixed_redirect() {
        let context = Context::test(());

        let b1 = BBox::new(String::from("hello"), NoPolicy {});
        let b2 = BBox::new(10u32, NoPolicy {});
        let b3 = -20i32;
        let b4 = "my_str";

        let redirect = BBoxRedirect::to("/test/{}/more/{}/{}/less/{}", (&b1, &b2, &b3, &b4,), context);
        let str = format!("{:?}", redirect.redirect);
        assert!(str.contains("\"/test/hello/more/10/-20/less/my_str\""));
    }
}