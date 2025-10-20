use std::collections::BTreeMap;

use sesame::bbox::BBox;
use sesame::policy::NoPolicy;
use sesame_rocket::bbox::{BBoxRender, Renderable};

use crate::application::policy::ACLPolicy;

pub struct Grade {
    pub id: BBox<u64, NoPolicy>,
    pub name: BBox<String, NoPolicy>,
    pub grade: BBox<u64, ACLPolicy>,
}

impl BBoxRender for Grade {
    fn render(&self) -> Renderable {
        Renderable::Dict(BTreeMap::from([
            (String::from("id"), self.id.render()),
            (String::from("name"), self.name.render()),
            (String::from("grade"), self.grade.render()),
        ]))
    }
}
