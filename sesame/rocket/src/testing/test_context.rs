use std::any::Any;

use sesame::testing::TestContextData;

use crate::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};

#[rocket::async_trait]
impl<'a, 'r, T: Send + Any> FromBBoxRequest<'a, 'r> for TestContextData<T> {
    type BBoxError = ();
    async fn from_bbox_request(
        _request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        todo!("TestContextData should not be actually constructed FromBBoxRequest because it is only used for testing")
    }
}
