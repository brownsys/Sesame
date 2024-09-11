extern crate alohomora;

use alohomora::bbox::{BBox, DirectBBox};
use alohomora::pure::PrivacyPureRegion;
use rand::Rng;
use rand::prelude::SliceRandom;
use std::time::Instant;
use alohomora::context::{Context, UnprotectedContext};
use alohomora::policy::{schema_policy, AnyPolicy, Policy, PolicyAnd, Reason, SchemaPolicy, NoPolicy};
use alohomora::AlohomoraType;
use std::sync::{Arc, Mutex};
use alohomora::pcr::{execute_pcr, PrivacyCriticalRegion};
use alohomora::testing::TestContextData;

#[derive(Clone)]
pub struct AnswerAccessPolicy {
    owner: Option<String>, // even if no owner, admins may access
    lec_id: Option<u64>,   // no lec_id when Policy for multiple Answers from different lectures
}

impl AnswerAccessPolicy {
    pub fn new(owner: Option<String>, lec_id: Option<u64>) -> AnswerAccessPolicy {
        AnswerAccessPolicy {
            owner: owner,
            lec_id: lec_id,
        }
    }
}

// Content of answer column can only be accessed by:
//   1. The user who submitted the answer (`user_id == me`);
//   2. The admin(s) (`is me in set<admins>`);
//   3. Any student who is leading discussion for the lecture
//      (`P(me)` alter. `is me in set<P(students)>`);
impl Policy for AnswerAccessPolicy {
    fn name(&self) -> String {
        format!(
            "AnswerAccessPolicy(lec id{:?} for user {:?})",
            self.lec_id, self.owner
        )
    }

    fn check(&self, context: &UnprotectedContext, _reason: Reason) -> bool {
        return true;
    }

    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!();
    }

    fn join_logic(&self, p2: Self) -> Result<Self, ()> {
        todo!();
    }
}

impl SchemaPolicy for AnswerAccessPolicy {
    fn from_row(_table: &str, row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized,
    {
        todo!();
    }
}

fn direct_bbox_vec() -> Vec<DirectBBox<u32, AnswerAccessPolicy>> {
    let mut bbox_vector = vec![];
    for n in 0..10000000 {
        let bbox = DirectBBox::new(n, AnswerAccessPolicy::new(Some("Sarah".to_string()), Some(0)));
        bbox_vector.push(bbox);
    }
    bbox_vector
}

fn bbox_vec() -> Vec<BBox<u32, AnswerAccessPolicy>> {
    //let mut v = vec![];
    let mut bbox_vector = vec![];
    for n in 0..10000000 {
        let bbox = BBox::new(n, AnswerAccessPolicy::new(Some("Sarah".to_string()), Some(0)));
        bbox_vector.push(bbox);
        //for i in 0..25 {
           //v.push(Box::new(n+i));
        //}
    }
    bbox_vector
}

fn random_index_vec() -> Vec<usize> {
    let mut rng = rand::thread_rng();
    (0..10000000).map(|_| rng.gen_range(0..10000000)).collect()
}

fn indirect_random() {
    let random_numbers = random_index_vec();
    let bbox_vector = bbox_vec();

    // Start timer before the second loop
    let start = Instant::now();

    // Access bboxes randomly to multiply value inside by 2
    let mut sum = 0;
    for i in 0..random_numbers.len() {
        let index = random_numbers[i];
        let bbox = &bbox_vector[index];
        let bbox = bbox.ppr(PrivacyPureRegion::new(|val: &u32| val * 2));
        // sum += bbox.to_owned_policy().discard_box();
        bbox.into_unbox(
            Context::new(String::from(""), TestContextData::new(())),
            PrivacyCriticalRegion::new(|val, _| {
                sum += val;
            }),
        ());
    }

    // Stop timer and print elapsed time
    let duration = start.elapsed();
    println!("{}", sum);
    println!("Random Indirect PCon accesses time: {:?}", duration);
    
    // Start timer before the second loop
    let start = Instant::now();
}

fn indirect_sequential() {
    let bbox_vector = bbox_vec();

    // Start timer before the second loop
    let start = Instant::now();

    // Access bboxes randomly to multiply value inside by 2
    let mut sum = 0;
    for i in 0..bbox_vector.len() {
        let bbox = &bbox_vector[i];
        let bbox = bbox.ppr(PrivacyPureRegion::new(|val: &u32| val * 2));
        // sum += bbox.to_owned_policy().discard_box();
        bbox.into_unbox(
            Context::new(String::from(""), TestContextData::new(())),
            PrivacyCriticalRegion::new(|val, _| {
                sum += val
            }),
        ());
    }

    // Stop timer and print elapsed time
    let duration = start.elapsed();
    println!("{}", sum);
    println!("Sequential Indirect PCon accesses time: {:?}", duration);
}

fn direct_random() {
    let random_numbers = random_index_vec();
    let bbox_vector = direct_bbox_vec();

    // Start timer before the second loop
    let start = Instant::now();

    // Access bboxes randomly to multiply value inside by 2
    let mut sum = 0;
    for i in 0..random_numbers.len() {
        let index = random_numbers[i];
        let bbox = &bbox_vector[index];
        let bbox = bbox.ppr(PrivacyPureRegion::new(|val: &u32| val * 2));
        // sum += bbox.to_owned_policy().discard_box();
        bbox.into_unbox(
            Context::new(String::from(""), TestContextData::new(())),
            PrivacyCriticalRegion::new(|val, _| {
                sum += val
            }),
        ());
    }

    // Stop timer and print elapsed time
    let duration = start.elapsed();
    println!("{}", sum);
    println!("Random Direct PCon accesses time: {:?}", duration);
}

fn direct_sequential() {
    let bbox_vector = direct_bbox_vec();

    // let context = Context::new(
    //     String::from(""),
    //     TestContextData::new(()),
    // );
    
    // Start timer before the second loop
    let start = Instant::now();

    // Access bboxes randomly to multiply value inside by 2
    let mut sum = 0;
    for i in 0..bbox_vector.len() {
        let bbox = &bbox_vector[i];
        let bbox = bbox.ppr(PrivacyPureRegion::new(|val: &u32| val * 2));
        // sum += bbox.to_owned_policy().discard_box();
        bbox.into_unbox(
            Context::new(String::from(""), TestContextData::new(())),
            PrivacyCriticalRegion::new(|val, _| {
                sum += val
            }),
        ());
    }

    // Stop timer and print elapsed time
    let duration = start.elapsed();
    println!("{}", sum);
    println!("Sequential Direct PCon accesses time: {:?}", duration);
}

fn main() {
  indirect_random();
  direct_random();
  indirect_sequential();
  direct_sequential();
}
