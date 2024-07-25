use websubmit_boxed::{make_rocket, parse_args};

#[rocket::main]
async fn main() {
    let args = parse_args();
    // println!("args are {:?}", args);
    let rocket = make_rocket(args);
    if let Err(e) = rocket.launch().await {
        println!("Whoops, didn't launch!");
        drop(e);
    };
}
