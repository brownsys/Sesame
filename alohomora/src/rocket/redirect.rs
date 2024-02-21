use crate::rocket::response::{BBoxResponseResult, BBoxResponder, BBoxResponse};
use crate::rocket::request::BBoxRequest;
use crate::rocket::redirect_parameters::RedirectParams;

use dynfmt::{Format, SimpleCurlyFormat};

// A redirect response.
pub struct BBoxRedirect {
    redirect: rocket::response::Redirect,
}
impl BBoxRedirect {
    pub fn to<'a, P: Into<RedirectParams>>(name: &str, params: P) -> Self {
        let params: RedirectParams = params.into();
        let formatted_str = SimpleCurlyFormat.format(name, params.parameters).unwrap();
        BBoxRedirect {
            redirect: rocket::response::Redirect::to(Into::<String>::into(formatted_str)),
        }
    }
}
impl<'r, 'o: 'r> BBoxResponder<'r, 'o> for BBoxRedirect {
    fn respond_to(self, request: &BBoxRequest<'r, '_>) -> BBoxResponseResult<'o> {
        use rocket::response::Responder;
        match self.redirect.respond_to(request.get_request()) {
            Ok(response) => Ok(BBoxResponse::new(response)),
            Err(e) => Err(e),
        }
    }
}

// Unit tests.
#[cfg(test)]
mod tests {
    use crate::bbox::BBox;
    use crate::policy::NoPolicy;
    use crate::rocket::BBoxRedirect;

    #[test]
    fn test_mixed_redirect() {
        let b1 = BBox::new(String::from("hello"), NoPolicy {});
        let b2 = BBox::new(10u32, NoPolicy {});
        let b3 = -20i32;
        let b4 = "my_str";

        let redirect = BBoxRedirect::to("/test/{}/more/{}/{}/less/{}", (&b1, &b2, &b3, &b4,));
        let str = format!("{:?}", redirect.redirect);
        assert!(str.contains("\"/test/hello/more/10/-20/less/my_str\""));
    }
}