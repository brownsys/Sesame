extern crate rocket;

use youchat::build_server;

#[rocket::main]
async fn main() {
    if let Err(e) = build_server()
        .launch()
        .await 
    {
        println!("didn't launch properly");
        drop(e);
    };
    
}