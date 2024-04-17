extern crate alohomora;
extern crate bench_lib;

use std::collections::HashSet;

use std::vec;

use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::pure::PrivacyPureRegion;
use alohomora::sandbox::execute_sandbox;

use bench_lib::{add_numbers, hash, mult_numbers, train, Numbers};

use chrono::naive::NaiveDateTime;
use linfa_linear::FittedLinearRegression;

fn main() {
    let bbox = BBox::new(Numbers { a: 4, b: 15 }, NoPolicy {});
    // START TIMER
    let bbox = execute_sandbox::<mult_numbers, _, _>(bbox);
    // END TIMER
    let bbox = bbox.specialize_policy::<NoPolicy>().unwrap();
    println!("{}", bbox.discard_box());

    // // PPR.
    // let set = HashSet::from([10u32, 7u32]);
    // let bbox = BBox::new(10u32, NoPolicy {});
    // let bbox = bbox.into_ppr(PrivacyPureRegion::new(|val| set.contains(&val)));
    // println!("{}", bbox.discard_box());

    // let bbox = BBox::new(5u32, NoPolicy {});
    // let bbox = bbox.into_ppr(PrivacyPureRegion::new(|val| {
    //   println!("Buggy leak {}", val);
    //   set.contains(&val)
    // }));
    // println!("{}", bbox.discard_box());

    let email = BBox::new("allen_aby@brown.edu".to_string(), NoPolicy {});
    let secret = BBox::new("SECRET".to_string(), NoPolicy {});
    // START TIMER (end inside hash)
    let hash = execute_sandbox::<hash, _, _>((email, secret));
    // END TIMER (start inside hash)
    let hash = hash.specialize_policy::<NoPolicy>().unwrap();
    println!("{}", hash.discard_box());

    type BBoxTime = BBox<NaiveDateTime, NoPolicy>;
    type BBoxGrade = BBox<u64, NoPolicy>;
    let grades: Vec<(BBoxTime, BBoxGrade)> = vec![
      (BBox::new(NaiveDateTime::parse_from_str("2023-03-13 13:40:26", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy {}), BBox::new(90 as u64, NoPolicy {})),
      (BBox::new(NaiveDateTime::parse_from_str("2023-03-09 13:54:05", "%Y-%m-%d %H:%M:%S").unwrap(), NoPolicy {}), BBox::new(95 as u64, NoPolicy {})),
    ];
    // START TIMER (end inside train)
    let model = execute_sandbox::<train, _, _>(grades);
    let time = NaiveDateTime::parse_from_str("2023-03-13 13:40:50", "%Y-%m-%d %H:%M:%S");
    // END TIMER (start inside train)
    let grade = model.into_ppr(PrivacyPureRegion::new(|model: FittedLinearRegression<f64>|
      model.params()[0] * (time.unwrap().and_utc().timestamp() as f64) + model.intercept()
    ));
    let grade = grade.specialize_policy::<NoPolicy>().unwrap();
    println!("{}", grade.discard_box());
}
