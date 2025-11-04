use std::any::Any;

use sesame::context::Context;
use sesame::pcon::PCon;
use sesame::policy::AnyPolicyDyn;
use sesame::{SesameType, SesameTypeEnum, SesameTypeOut};

use sesame_rocket::rocket::{FromPConRequest, PConCookie, PConRequest, PConRequestOutcome};

use rocket::async_trait;

use crate::application::policy::AuthenticationCookiePolicy;

// Application specific portion of context.
#[derive(Clone)]
pub struct ContextData {
    user: Option<PCon<String, AuthenticationCookiePolicy>>,
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
            user: Option::<PCon<String, AuthenticationCookiePolicy>>::from_enum(e)?,
        })
    }
    fn out_from_enum(e: SesameTypeEnum<dyn Any, dyn AnyPolicyDyn>) -> Result<Self::Out, ()> {
        Option::<PCon<String, AuthenticationCookiePolicy>>::out_from_enum(e)
    }
}

// Can be loaded from application.
#[async_trait]
impl<'a, 'r> FromPConRequest<'a, 'r> for ContextData {
    type PConError = ();

    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        let cookie: Option<PConCookie<AuthenticationCookiePolicy>> = request.cookies().get("user");
        let user = cookie.map(|cookie| cookie.value().to_owned_policy().into_pcon());
        PConRequestOutcome::Success(ContextData { user })
    }
}
