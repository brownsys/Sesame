extern crate erased_serde;
extern crate figment;

use std::borrow::Cow;
use std::result::Result;

// Our BBox struct.
use crate::bbox::BBoxRender;
use crate::policy::Context;
use crate::rocket::request::BBoxRequest;
use crate::rocket::response::{BBoxResponder, BBoxResponse, BBoxResponseResult};

pub struct BBoxTemplate {
    template: rocket_dyn_templates::Template,
}

impl BBoxTemplate {
    // Our render wrapper takes in some BBoxRender type, transforms it to a figment
    // Value compatible with Rocket, and then calls Rocket's render.
    pub fn render<S: Into<Cow<'static, str>>, T: BBoxRender, U: 'static, D: 'static>(
        name: S,
        params: &T,
        context: &Context<U, D>,
    ) -> Result<Self, figment::Error> {
        // First turn context into a figment::value::Value.
        let transformed = params.render().transform(context)?;
        // Now render.
        let template = rocket_dyn_templates::Template::render(name, transformed);
        Ok(BBoxTemplate { template })
    }
}

impl<'r, 'o: 'r> BBoxResponder<'r, 'o> for BBoxTemplate {
    fn respond_to(self, request: &BBoxRequest<'r, '_>) -> BBoxResponseResult<'o> {
        use rocket::response::Responder;
        match self.template.respond_to(request.get_request()) {
            Result::Ok(response) => Result::Ok(BBoxResponse::new(response)),
            Result::Err(e) => Result::Err(e),
        }
    }
}
