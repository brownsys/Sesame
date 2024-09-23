use rocket::async_trait;
use alohomora::{AlohomoraType, Unwrapper};
use alohomora::bbox::BBox;
use alohomora::context::Context;
use alohomora::policy::OptionPolicy;
use alohomora::rocket::{BBoxCookie, BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};

use crate::application::policy::AuthenticationCookiePolicy;

// Application specific portion of context.
#[derive(Clone)]
pub struct ContextData {
    user: Option<BBox<String, AuthenticationCookiePolicy>>,
}

// Application contexts.
pub type AppContext = Context<ContextData>;

impl AlohomoraType for ContextData {
    type Out = Option<String>;
    type Policy = OptionPolicy<AuthenticationCookiePolicy>;

    fn inner_fold(self, unwrapper: &Unwrapper) -> Result<(Self::Out, Self::Policy), ()> {
        self.user.inner_fold(unwrapper)
    }
}

// Can be loaded from application.
#[async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for ContextData {
    type BBoxError = ();

    async fn from_bbox_request(request: BBoxRequest<'a, 'r>) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let cookie: Option<BBoxCookie<AuthenticationCookiePolicy>> = request.cookies().get("user");
        let user = cookie.map(|cookie| {
            cookie.value().to_owned_policy().into_bbox()
        });
        BBoxRequestOutcome::Success(ContextData { user })
    }
}