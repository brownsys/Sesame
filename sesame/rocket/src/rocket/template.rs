extern crate erased_serde;
extern crate figment;

use std::borrow::Cow;
use std::ops::Deref;
use std::result::Result;

use sesame::context::{Context, ContextData};
use sesame::extensions::ExtensionContext;
// Our BBox struct.
use crate::error::SesameRenderResult;
use crate::render::BBoxRender;
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
    ) -> SesameRenderResult<Self> {
        let name = name.into();
        // First turn context into a figment::value::Value.
        let context = ExtensionContext::new(context);
        let transformed = params.render().transform(name.deref(), &context)?;
        // Now render.
        let template = rocket_dyn_templates::Template::render(name, transformed);
        Ok(BBoxTemplate { template })
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
