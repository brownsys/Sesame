extern crate alohomora;

use alohomora::bbox::{BBox, DirectBBox};
use alohomora::policy::NoPolicy;
use alohomora::pure::PrivacyPureRegion;
use rand::Rng;
use rand::prelude::SliceRandom;
use std::time::Instant;


fn direct_bbox_vec() -> Vec<DirectBBox<u32, NoPolicy>> {
    let mut bbox_vector = vec![];
    for n in 0..10000000 {
        let bbox = DirectBBox::new(n, NoPolicy {});
        bbox_vector.push(bbox);
    }
    bbox_vector
}

fn bbox_vec() -> Vec<BBox<u32, NoPolicy>> {
    //let mut v = vec![];
    let mut bbox_vector = vec![];
    for n in 0..10000000 {
        let bbox = BBox::new(n, NoPolicy {});
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
        sum += bbox.to_owned_policy().discard_box();
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
        sum += bbox.to_owned_policy().discard_box();
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
        sum += bbox.to_owned_policy().discard_box();
    }

    // Stop timer and print elapsed time
    let duration = start.elapsed();
    println!("{}", sum);
    println!("Random Direct PCon accesses time: {:?}", duration);
}

fn direct_sequential() {
    let bbox_vector = direct_bbox_vec();
    
    // Start timer before the second loop
    let start = Instant::now();

    // Access bboxes randomly to multiply value inside by 2
    let mut sum = 0;
    for i in 0..bbox_vector.len() {
        let bbox = &bbox_vector[i];
        let bbox = bbox.ppr(PrivacyPureRegion::new(|val: &u32| val * 2));
        sum += bbox.to_owned_policy().discard_box();
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
