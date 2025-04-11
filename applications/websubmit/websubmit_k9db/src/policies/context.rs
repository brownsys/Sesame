use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rocket::State;

use alohomora::bbox::BBox;
use alohomora::db::from_value;
use alohomora::k9db::context::{K9dbContext, K9dbContextData, K9dbContextDataTrait};
use alohomora::policy::{AnyPolicy, NoPolicy};
use alohomora::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};

use crate::backend::MySqlBackend;
use crate::policies::QueryableOnly;

// Defines purposes
const PURPOSE_MAP: [(&'static str, &'static str); 2] = [
    ("<ML_EXPERIMENT>", "ml_experiment"),
    ("<EMPLOYERS>", "employers"),
];

// Custom developer defined payload attached to every context.
#[derive(Clone)]
pub struct WebsubmitContextData {
    pub user: BBox<Option<String>, QueryableOnly>,
    pub purpose: BBox<Option<String>, QueryableOnly>,
}

pub type ContextData = K9dbContextData<WebsubmitContextData>;
pub type Context = K9dbContext<WebsubmitContextData>;

// Compatible with K9db policies.
impl K9dbContextDataTrait for WebsubmitContextData {
    fn user(&self) -> BBox<Option<String>, AnyPolicy> {
        self.user.clone().into_any_policy()
    }
    fn purpose(&self) -> BBox<Option<String>, AnyPolicy> {
        self.purpose.clone().into_any_policy()
    }
}

// Build the custom payload for the context given HTTP request.
#[rocket::async_trait]
impl<'a, 'r> FromBBoxRequest<'a, 'r> for WebsubmitContextData {
    type BBoxError = ();

    async fn from_bbox_request(
        request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let db: &State<Arc<Mutex<MySqlBackend>>> = request.guard().await.unwrap();

        // Find user using ApiKey token from cookie.
        let apikey = request.cookies().get::<QueryableOnly>("apikey");
        let user = match apikey {
            None => BBox::new(None, QueryableOnly {}),
            Some(apikey) => {
                let apikey = apikey.value().to_owned();
                let mut bg = db.lock().unwrap();
                let res = bg.prep_exec(
                    "SELECT * FROM users WHERE apikey = ?",
                    (apikey,),
                    alohomora::context::Context::empty(),
                );
                drop(bg);
                if res.len() > 0 {
                    let user: BBox<String, NoPolicy> = from_value(res[0][0].clone()).unwrap();
                    BBox::new(Some(user.discard_box()), QueryableOnly {})
                } else {
                    BBox::new(None, QueryableOnly {})
                }
            }
        };

        // Find purpose by looking at routes.
        let path = request.path();
        println!("{:?}", path);

        let mut purpose = None;
        for (purpose_path, purpose_str) in PURPOSE_MAP {
            if path == purpose_path {
                purpose = Some(String::from(purpose_str));
                break;
            }
        }
        let purpose = BBox::new(purpose, QueryableOnly {});
        BBoxRequestOutcome::Success(WebsubmitContextData { user, purpose })
    }
}