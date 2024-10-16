use alohomora::bbox::BBox;
use alohomora::context::Context;
use alohomora::policy::{Policy, AnyPolicy, NoPolicy};

use alohomora::db::{BBoxConn, BBoxOpts, BBoxParams, BBoxStatement, BBoxValue};

use unsafelib::{log_malicious, log};

fn connect_db() -> BBoxConn {
  let mut db = BBoxConn::new(
    // this is the user and password from the config.toml file
    BBoxOpts::from_url(&format!("mysql://{}:{}@127.0.0.1/", "root", "password")).unwrap(),
  ).unwrap();
  assert_eq!(db.ping(), true);

  let schema = std::fs::read_to_string("schema.sql").unwrap();
  for line in schema.lines() {
    if line.starts_with("--") || line.is_empty() {
        continue;
    }
    db.query_drop(line).unwrap();
  }

  db
}


pub fn check_api_key<P: Policy + Clone + 'static>(
    backend: &mut BBoxConn,
    apikey: BBox<String, P>,
    context: Context<()>,
) -> Result<BBox<String, AnyPolicy>, ()> {
    log_malicious::<_, String>(&apikey);

    let result = backend.exec_iter(
        "SELECT * FROM users WHERE apikey = ?",
        (apikey,),
        context,
    );
    
    match result {
      Err(_) => Err(()),
      Ok(mut result) => match result.next() {
        None => Err(()),
        Some(row) => Ok(row.unwrap().get(0).unwrap()),
      }
    }
}

pub fn main() {
  // Connect to the DB.
  let mut db: BBoxConn = connect_db();
  let apikey1 = BBox::new(String::from("123456789"), NoPolicy {});
  let apikey2 = BBox::new(String::from("555666999"), NoPolicy {});
  println!("{}", check_api_key(&mut db, apikey1, Context::empty()).is_ok());
  println!("{}", check_api_key(&mut db, apikey2, Context::empty()).is_ok());
}
