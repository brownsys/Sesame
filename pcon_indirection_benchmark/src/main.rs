extern crate alohomora;

use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::pure::PrivacyPureRegion;
use rand::Rng;
use std::time::Instant;

fn main() {
    // Make vector of 1,000,000 random numbers
    let mut rng = rand::thread_rng();
    let random_numbers: Vec<usize> = (0..10000000)
        .map(|_| rng.gen_range(0..10000000))
        .collect();

    // Make vector of 10,000,000 bboxes
    let mut bbox_vector = vec![];
    for n in 0..10000000 {
        let bbox = BBox::new(n, NoPolicy {});
        bbox_vector.push(bbox)
    }

    // Start timer before the second loop
    let start = Instant::now();

    // Access bboxes randomly to multiply value inside by 2
    for index in random_numbers.iter() {
        let bbox = bbox_vector[*index].clone();
        bbox_vector[*index] = bbox.into_ppr(PrivacyPureRegion::new(|val: u32| {
            val * 2
        }));
    }

    // Stop timer and print elapsed time
    let duration = start.elapsed();
    println!("PCon accesses time: {:?}", duration);
}
