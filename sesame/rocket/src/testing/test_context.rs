use std::any::Any;

use sesame::testing::TestContextData;

use crate::rocket::{FromPConRequest, PConRequest, PConRequestOutcome};

#[rocket::async_trait]
impl<'a, 'r, T: Send + Any> FromPConRequest<'a, 'r> for TestContextData<T> {
    type PConError = ();
    async fn from_pcon_request(
        _request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        todo!("TestContextData should not be actually constructed FromPConRequest because it is only used for testing")
    }
}
