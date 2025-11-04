/// Some rocket specific implementations to bridge SeaORM with rocket.
use sesame_rocket::rocket::{FromPConRequest, PConRequest, PConRequestOutcome};

// We should use these structs along with their derive macros regularly when using
// Sesame + (SeaORM and Rocket), with only one difference.
// When implementing Pool, the type Connection should be set to sesame_orm::PConDatabaseConnection
// instead of sea_orm::DatabaseConnection.
pub use sea_orm_rocket::{Connection, Database, Pool};

// This allows us to use Connection as a guard in routes.
#[rocket::async_trait]
impl<'a, 'r, D: sea_orm_rocket::Database> FromPConRequest<'a, 'r> for Connection<'a, D> {
    type PConError = ();

    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        use sea_orm_rocket::rocket::request::FromRequest;
        match Connection::from_request(request.get_request()).await {
            PConRequestOutcome::Success(result) => PConRequestOutcome::Success(result),
            PConRequestOutcome::Forward(forward) => PConRequestOutcome::Forward(forward),
            PConRequestOutcome::Failure((status, _)) => PConRequestOutcome::Failure((status, ())),
        }
    }
}
