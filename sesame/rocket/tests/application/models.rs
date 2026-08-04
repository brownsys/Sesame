use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame_rocket::render::{PConRender, Renderable};

use crate::application::policy::ACLPolicy;

pub struct Grade {
    pub id: PCon<u64, NoPolicy>,
    pub name: PCon<String, NoPolicy>,
    pub grade: PCon<u64, ACLPolicy>,
}

impl PConRender for Grade {
    fn render(&self) -> Renderable {
        Renderable::Object(Vec::from([
            ("id", self.id.render()),
            ("name", self.name.render()),
            ("grade", self.grade.render()),
        ]))
    }
}
