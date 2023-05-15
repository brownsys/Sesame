use crate::config::Config;
use rocket::State;
use rocket_dyn_templates::Template;
use std::collections::HashMap;
use bbox::BBox;
use bbox::render::render_boxed;

#[get("/")]
pub(crate) fn login(config: &State<Config>) -> Template {
    let mut ctx = HashMap::new();

    let classid = BBox::new(config.class.clone());
    let parent = BBox::new(String::from("layout"));

    let try_serialized = serde_json::to_string(&classid).unwrap();
    println!("{try_serialized}");

    ctx.insert("CLASS_ID", classid);
    ctx.insert("parent", parent);

    render_boxed("login", &ctx)
}
