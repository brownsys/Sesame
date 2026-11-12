use std::sync::Arc;

use rocket::State;
use sesame::SesameType;
use sesame::context::Context;
use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame::verified::VerifiedRegion;
use sesame_rocket::rocket::{FromPConRequest, PConRequest, PConRequestOutcome};

use crate::database::Database;
use crate::policy::ViewUserPolicy;

#[derive(SesameType, Clone)]
#[sesame_out_type(verbatim = [db])]
pub struct ContextData {
    pub user_id: Option<PCon<i32, ViewUserPolicy>>, // None when not logged in
    pub admin: Option<PCon<bool, ViewUserPolicy>>,
    pub balance_cents: Option<PCon<i64, ViewUserPolicy>>,
    pub db: Arc<Database>,
}

#[rocket::async_trait]
impl<'a, 'r> FromPConRequest<'a, 'r> for ContextData {
    type PConError = String;

    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        let db: &State<Arc<Database>> = request.guard().await.unwrap();

        // Ids double as api keys, so the cookie carries no policy. The
        // requester's fields come from their user row instead, and keep
        // that row's ViewUserPolicy.
        let user = request
            .cookies()
            .get::<NoPolicy>("user_id")
            .and_then(|cookie| {
                let id: PCon<String, NoPolicy> = cookie.into();
                id.into_verified(VerifiedRegion::new(|id: String| id.parse::<i32>().ok()))
                    .fold_in()
            })
            .and_then(|id| db.get_user_by_id(id, Context::empty()));

        let (user_id, admin, balance_cents) = match user {
            Some(user) => (Some(user.id), Some(user.admin), Some(user.balance_cents)),
            None => (None, None, None),
        };

        PConRequestOutcome::Success(ContextData {
            user_id,
            admin,
            balance_cents,
            db: db.inner().clone(),
        })
    }
}
