extern crate alohomora;

use std::collections::HashSet;

use alohomora::bbox::BBox;
use alohomora::policy::NoPolicy;
use alohomora::pure::PrivacyPureRegion;
use alohomora::pcr::PrivacyCriticalRegion;

fn main() {
    // Make vector of 1,000,000 bboxes
    let mut bbox_vector = vec![];
    for n in 0..999999 {
        let bbox = BBox::new(n, NoPolicy {});
        bbox_vector.push(bbox)
    }
    // Update each bbox to multiply value inside by 2
    for bbox in bbox_vector.iter_mut() {
        *bbox = bbox.clone().into_ppr(PrivacyPureRegion::new(|val: u32| {
            val * 2
        }));
    }
}
