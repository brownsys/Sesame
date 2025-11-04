use std::collections::BTreeMap;

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
        Renderable::Dict(BTreeMap::from([
            (String::from("id"), self.id.render()),
            (String::from("name"), self.name.render()),
            (String::from("grade"), self.grade.render()),
        ]))
    }
}
