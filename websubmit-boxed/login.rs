use std::collections::HashMap;

use rocket::State;
use rocket_dyn_templates::Template;

use bbox::BBox;
use bbox::context;
use bbox::render::render_boxed;

use crate::config::Config;

// #[get("/")]
// pub(crate) fn login(config: &State<Config>) -> Template {
//     let mut ctx = HashMap::new();
//
//     let id = BBox::new(config.class.clone());
//     let parent = BBox::new(String::from("layout"));
//
//     let try_serialized = serde_json::to_string(&id).unwrap();
//     println!("{try_serialized}");
//
//     ctx.insert("id", id);
//     ctx.insert("parent", parent);
//
//     render_boxed("login", &ctx)
// }

#[get("/")]
pub(crate) fn login(config: &State<Config>) -> Template {
  let id = BBox::new(config.class.clone());
  let parent = BBox::new(String::from("layout"));

  let try_serialized = serde_json::to_string(&id).unwrap();
  println!("{try_serialized}");

  Template::render("login", context! {id: &id, parent: &parent})
}
