extern crate erased_serde;
extern crate figment;

use std::borrow::Cow;
use std::ops::Deref;
use std::result::Result;

// Our BBox struct.
use crate::bbox::BBoxRender;
use crate::context::{Context, ContextData, UnprotectedContext};
use crate::rocket::request::BBoxRequest;
use crate::rocket::response::{BBoxResponder, BBoxResponse, BBoxResponseResult};

pub struct BBoxTemplate {
    template: rocket_dyn_templates::Template,
}

impl BBoxTemplate {
    // Our render wrapper takes in some BBoxRender type, transforms it to a figment
    // Value compatible with Rocket, and then calls Rocket's render.
    pub fn render<S: Into<Cow<'static, str>>, T: BBoxRender, D: ContextData>(
        name: S,
        params: &T,
        context: Context<D>,
    ) -> Self {
        let name = name.into();
        // First turn context into a figment::value::Value.
        let context = UnprotectedContext::from(context);
        let transformed = params.render().transform(name.deref(), &context).unwrap();
        // Now render.
        let template = rocket_dyn_templates::Template::render(name, transformed);
        BBoxTemplate { template }
    }
}

impl<'a, 'r> BBoxResponder<'a, 'r, 'static> for BBoxTemplate {
    fn respond_to(self, request: BBoxRequest<'a, 'r>) -> BBoxResponseResult<'static> {
        match rocket::response::Responder::respond_to(self.template, request.get_request()) {
            Result::Ok(response) => Result::Ok(BBoxResponse::new(response)),
            Result::Err(e) => Result::Err(e),
        }
    }
}
