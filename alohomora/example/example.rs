use rocket::{FromForm, get, routes};

#[derive(FromForm)]
pub struct Dog {
    a: String,
    b: u32,
}

#[get("/route/<num>?<dog>&<a>")]
pub fn route(num: u32, a: String, dog: Dog) {}



fn main() {
    rocket::build().mount("/", routes![route]);
}