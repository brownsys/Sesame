extern crate erased_serde;
extern crate figment;

use std::borrow::Cow;
use std::ops::Deref;
use std::result::Result;

use sesame::context::{Context, ContextData};
use sesame::extensions::ExtensionContext;

use crate::error::SesameRenderResult;
use crate::render::PConRender;
use crate::rocket::request::PConRequest;
use crate::rocket::response::{PConResponder, PConResponse, PConResponseResult};

pub struct PConTemplate {
    template: rocket_dyn_templates::Template,
}

impl PConTemplate {
    // Our render wrapper takes in some PConRender type, transforms it to a figment
    // Value compatible with Rocket, and then calls Rocket's render.
    pub fn render<S: Into<Cow<'static, str>>, T: PConRender, D: ContextData>(
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
        Ok(PConTemplate { template })
    }
}

impl<'a, 'r> PConResponder<'a, 'r, 'static> for PConTemplate {
    fn respond_to(self, request: PConRequest<'a, 'r>) -> PConResponseResult<'static> {
        match rocket::response::Responder::respond_to(self.template, request.get_request()) {
            Result::Ok(response) => Result::Ok(PConResponse::new(response)),
            Result::Err(e) => Result::Err(e),
        }
    }
}
