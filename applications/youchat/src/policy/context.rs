use std::sync::{Arc, Mutex};

use rocket::State;
use sesame::context::Context;
use sesame::SesameType;
use sesame::{pcon::PCon, policy::NoPolicy};
use sesame_mysql::{PConOpts, SesameConn};
use sesame_rocket::rocket::{FromPConRequest, PConRequest, PConRequestOutcome};

use crate::config::Config;

// the actual context data (just our backend)
#[derive(SesameType)]
#[sesame_out_type(verbatim = [config])]
pub struct ContextData {
    pub user: Option<PCon<String, NoPolicy>>,
    pub db: Arc<Mutex<SesameConn>>,
    pub config: Config,
}
impl Clone for ContextData {
    fn clone(&self) -> Self {
        let mut db = SesameConn::new(
            // this is the user and password from the config.toml file
            PConOpts::from_url(&format!(
                "mysql://{}:{}@127.0.0.1/",
                self.config.db_user, self.config.db_password
            ))
            .unwrap(),
        )
        .unwrap();
        db.query_drop("USE chats").unwrap();

        Self {
            user: self.user.clone(),
            db: Arc::new(Mutex::new(db)),
            config: self.config.clone(),
        }
    }
}

pub type YouChatContext = Context<ContextData>;

// Build the custom payload for the context given HTTP request.
// adapted from websubmit
#[rocket::async_trait]
impl<'a, 'r> FromPConRequest<'a, 'r> for ContextData {
    type PConError = ();

    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        let config: &State<Config> = request.guard().await.unwrap();

        // Connect to the DB.
        let mut db = SesameConn::new(
            // this is the user and password from the config.toml file
            PConOpts::from_url(&format!(
                "mysql://{}:{}@127.0.0.1/",
                config.db_user, config.db_password
            ))
            .unwrap(),
        )
        .unwrap();
        db.query_drop("USE chats").unwrap();

        //from OG YouChat - params being segments from url
        let user = {
            match request.param(1) {
                // if the second param exists, that'll be the name
                Some(param2) => param2.unwrap(),
                // if not, the first param will be the name if we have one
                None => match request.param(0) {
                    Some(param1) => Some(param1.unwrap()),
                    None => None,
                },
            }
        };

        PConRequestOutcome::Success(ContextData {
            user: user,
            db: Arc::new(Mutex::new(db)),
            config: config.inner().clone(),
        })
    }
}
