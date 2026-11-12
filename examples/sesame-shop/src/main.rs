mod context;
mod controller;
mod database;
mod model;
mod policy;

use std::sync::Arc;

use sesame_rocket::rocket::{routes, SesameRocket};

use crate::database::Database;

#[rocket::main]
async fn main() {
    let db = Database::new("mysql://root:password@127.0.0.1:3306/shop");
    db.create_tables_if_not_exists();

    let result = SesameRocket::build()
        .manage(Arc::new(db))
        .mount(
            "/",
            routes![
                controller::register,
                controller::login,
                controller::get_items,
                controller::sell,
                controller::purchase,
                controller::get_user_purchases,
                controller::get_item_purchases,
            ],
        )
        .launch()
        .await;

    if let Err(e) = result {
        eprintln!("rocket failed to launch: {e}");
    }
}
