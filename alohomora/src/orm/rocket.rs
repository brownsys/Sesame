/// Some rocket specific implementations to bridge SeaORM with rocket.
use crate::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};

// We should use these structs along with their derive macros regularly when using
// Alohomora + (SeaORM and Rocket), with only one difference.
// When implementing Pool, the type Connection should be set to alohomora::orm::BBoxDatabaseConnection
// instead of sea_orm::DatabaseConnection.
pub use sea_orm_rocket::{Database, Pool, Connection};

// This allows us to use Connection as a guard in routes.
#[rocket::async_trait]
impl<'a, 'r, D: sea_orm_rocket::Database> FromBBoxRequest<'a, 'r> for Connection<'a, D> {
    type BBoxError = ();

    async fn from_bbox_request(request: BBoxRequest<'a, 'r>) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        use sea_orm_rocket::rocket::request::FromRequest;
        match Connection::from_request(request.get_request()).await {
            BBoxRequestOutcome::Success(result) => BBoxRequestOutcome::Success(result),
            BBoxRequestOutcome::Forward(forward) => BBoxRequestOutcome::Forward(forward),
            BBoxRequestOutcome::Failure((status, _)) => BBoxRequestOutcome::Failure((status, ())),
        }
    }
}