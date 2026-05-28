use rocket::async_trait;
use sesame::context::Context;
use sesame::SesameType;
use sesame_rocket::rocket::{FromPConRequest, PConRequest, PConRequestOutcome};

#[derive(SesameType, Clone)]
pub struct RenderContextData {}

pub type RenderContext = Context<RenderContextData>;

#[async_trait]
impl<'a, 'r> FromPConRequest<'a, 'r> for RenderContextData {
    type PConError = ();

    async fn from_pcon_request(
        _request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        PConRequestOutcome::Success(RenderContextData {})
    }
}
