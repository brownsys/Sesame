use std::sync::{Arc, Mutex};
use alohomora::AlohomoraType;
use alohomora::{bbox::BBox, policy::NoPolicy};
use alohomora::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use alohomora::context::Context;
use alohomora::db::{BBoxConn, BBoxOpts};
use rocket::State;
use crate::config::Config; 

// the actual context data (just our backend)
#[derive(AlohomoraType)]
#[alohomora_out_type(verbatim = [config])]
pub struct ContextData {
    pub user: Option<BBox<String, NoPolicy>>,
    pub db: Arc<Mutex<BBoxConn>>,
    pub config: Config,
}
impl Clone for ContextData {
    fn clone(&self) -> Self {
        let mut db = BBoxConn::new(
            // this is the user and password from the config.toml file
            BBoxOpts::from_url(&format!("mysql://{}:{}@127.0.0.1/", self.config.db_user, self.config.db_password)).unwrap(),
        ).unwrap();
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
impl<'a, 'r> FromBBoxRequest<'a, 'r> for ContextData {
    type BBoxError = ();

    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let config: &State<Config> = request.guard().await.unwrap();

        // Connect to the DB.
        let mut db = BBoxConn::new(
            // this is the user and password from the config.toml file
            BBoxOpts::from_url(&format!("mysql://{}:{}@127.0.0.1/", config.db_user, config.db_password)).unwrap(),
        ).unwrap();
        db.query_drop("USE chats").unwrap();

        //from OG YouChat - params being segments from url
        let user = {
            match request.param(1)  {
                // if the second param exists, that'll be the name
                Some(param2) => param2.unwrap(),
                // if not, the first param will be the name if we have one
                None => match request.param(0) {
                    Some(param1) => Some(param1.unwrap()),
                    None => None,
                }
            }
        };

        BBoxRequestOutcome::Success(ContextData {
            user: user,
            db: Arc::new(Mutex::new(db)),
            config: config.inner().clone(),
        })
    }
}
