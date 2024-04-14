use std::sync::{Arc, Mutex};
use alohomora::AlohomoraType;
use alohomora::{bbox::BBox, policy::NoPolicy};
use alohomora::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use alohomora::context::Context; 
use rocket::{State, http::Status};
use rocket::outcome::IntoOutcome;
use crate::backend::MySqlBackend;
use crate::config::Config; 

// the actual context data (just our backend)
#[derive(AlohomoraType, Clone)]
#[alohomora_out_type(verbatim = [db, config])] 
pub struct ContextData {
    pub user: Option<BBox<String, NoPolicy>>, // TODO(corinn) is this actually NoPolicy?
    pub db: Arc<Mutex<MySqlBackend>>,
    pub config: Config,
}

pub type YouChatContext = Context<ContextData>; 

// Build the custom payload for the context given HTTP request.
// adapted from websubmit
#[rocket::async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for ContextData {
    type BBoxError = (); //TODO(corinn) use ContextDataError ?

    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let db: &State<Arc<Mutex<MySqlBackend>>> = request.guard().await.unwrap();
        let config: &State<Config> = request.guard().await.unwrap();


        //from OG YouChat - params being segments from url
        let user = {
            match request.param(1)  {
                // if the second param exists, that'll be the name
                Some(param2) => {
                    let boxed_name: BBox<_, NoPolicy> = param2.clone().unwrap();
                    let name: String = boxed_name.discard_box();
                    name.to_string().clone()
                },
                // if not, the first param will be the name if we have one
                None => {
                    match request.param(0) {
                        Some(param1) => {
                            let a: BBox<String, NoPolicy> = param1.clone().unwrap();
                            a.discard_box().to_string().clone()
                        }
                        None => "".to_string()
                    }
                }
            }
        };

        request
            .route()
            .and_then(|_| {
                Some(ContextData {
                    user: Some(BBox::new(user, NoPolicy::new())),
                    db: db.inner().clone(),
                    config: config.inner().clone(),
                })
            })
            .into_outcome((
                Status::InternalServerError,
                (),
            ))
    }
}
