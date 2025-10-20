use std::any::Any;

use sesame::bbox::BBox;
use sesame::context::Context;
use sesame::policy::AnyPolicyDyn;
use sesame::{SesameType, SesameTypeEnum, SesameTypeOut};

use sesame_rocket::rocket::{BBoxCookie, BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};

use rocket::async_trait;

use crate::application::policy::AuthenticationCookiePolicy;

// Application specific portion of context.
#[derive(Clone)]
pub struct ContextData {
    user: Option<BBox<String, AuthenticationCookiePolicy>>,
}

// Application contexts.
pub type AppContext = Context<ContextData>;

impl SesameTypeOut for ContextData {
    type Out = Option<String>;
}
impl SesameType for ContextData {
    fn to_enum(self) -> SesameTypeEnum {
        self.user.to_enum()
    }
    fn from_enum(e: SesameTypeEnum) -> Result<Self, ()> {
        Ok(ContextData {
            user: Option::<BBox<String, AuthenticationCookiePolicy>>::from_enum(e)?,
        })
    }
    fn out_from_enum(e: SesameTypeEnum<dyn Any, dyn AnyPolicyDyn>) -> Result<Self::Out, ()> {
        Option::<BBox<String, AuthenticationCookiePolicy>>::out_from_enum(e)
    }
}

// Can be loaded from application.
#[async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for ContextData {
    type BBoxError = ();

    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let cookie: Option<BBoxCookie<AuthenticationCookiePolicy>> = request.cookies().get("user");
        let user = cookie.map(|cookie| cookie.value().to_owned_policy().into_bbox());
        BBoxRequestOutcome::Success(ContextData { user })
    }
}
