use websubmit::{make_rocket, parse_args};

#[rocket::main]
async fn main() {
    let args = parse_args();
    let rocket = make_rocket(args);
    if let Err(e) = rocket.launch().await {
        println!("Whoops, didn't launch!");
        drop(e);
    };
}
