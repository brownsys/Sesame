
use data_policy::CandidateDataPolicy;
use alohomora::{bbox::BBox, pcr::{execute_pcr, PrivacyCriticalRegion, Signature}, AlohomoraType};
use alohomora_derive::ResponseBBoxJson;
use chrono::{Days, NaiveDateTime};
//use sea_orm::sea_query::private;
// use serde::{Deserialize, Serialize};
use std::{collections::HashMap, time::{Duration, Instant}};
use alohomora::policy::{AnyPolicy, NoPolicy};

mod data_policy;

#[derive(AlohomoraType, ResponseBBoxJson, Debug, Clone)]
#[alohomora_out_type(to_derive = [ResponseBBoxJson, Debug])]
pub struct ApplicationResponse {
    pub application_id: BBox<i32, AnyPolicy>,
    pub candidate_id: BBox<i32, AnyPolicy>,
    pub related_applications: Vec<BBox<i32, AnyPolicy>>,
    pub personal_id_number: BBox<String, AnyPolicy>,
    pub name: BBox<String, AnyPolicy>,
    pub surname: BBox<String, AnyPolicy>,
    pub email: BBox<String, AnyPolicy>,
    pub telephone: BBox<String, AnyPolicy>,
    pub field_of_study: Option<BBox<String, AnyPolicy>>,
    pub created_at: BBox<NaiveDateTime, AnyPolicy>,
}

fn compute_times(mut times: Vec<Duration>) -> (u64, u64, u64) {
    times.sort();
    let median = times[times.len() / 2].as_micros() as u64;
    let ninty = times[times.len() * 95 / 100].as_micros() as u64;
    let avg = times.iter().map(|t| t.as_micros() as u64).sum::<u64>() / times.len() as u64;
    (median, ninty, avg)
}

fn create_response(cand_id: Option<i32>) -> ApplicationResponse {
    let policy = AnyPolicy::new(CandidateDataPolicy::new(cand_id));
    ApplicationResponse { 
        application_id: BBox::new(0, AnyPolicy::new(policy.clone())), 
        candidate_id: BBox::new(cand_id.unwrap_or(0), AnyPolicy::new(policy.clone())), 
        related_applications: vec![BBox::new(3, policy.clone()), BBox::new(4, policy.clone()), BBox::new(5, policy.clone()), BBox::new(7, policy.clone())], 
        personal_id_number: BBox::new(format!("1234{:?}", cand_id), policy.clone()), 
        name: BBox::new(format!("roberto{:?}", cand_id), AnyPolicy::new(policy.clone())), 
        surname: BBox::new(format!("smith{:?}", cand_id), AnyPolicy::new(policy.clone())), 
        email: BBox::new(format!("roberto{:?}@smith.com", cand_id), AnyPolicy::new(policy.clone())), 
        telephone: BBox::new(format!("+1 902 283 2201/{:?}", cand_id), AnyPolicy::new(policy.clone())), 
        field_of_study: Some(BBox::new(format!("computers/{:?}", cand_id), policy.clone())), 
        created_at: BBox::new(NaiveDateTime::parse_from_str("2015-07-01 08:59:60.123", "%Y-%m-%d %H:%M:%S%.f").unwrap().checked_add_days(Days::new(cand_id.unwrap() as u64)).unwrap(), AnyPolicy::new(NoPolicy::new())) 
    }
}

const APPLICATION_COUNT: i32 = 1000;
const FOLDS: i32 = 100;


fn main() {
    // let vec = ;
    let vec = (0..APPLICATION_COUNT).into_iter().map(|id|{
        create_response(Some(id))
    }).collect::<Vec<ApplicationResponse>>();
    println!("vec built w/ len {:?}", vec);

    let mut times = vec![];
    for fold in 0..FOLDS {
        let vec_to_fold = vec.clone();
        let t = Instant::now();
        let vec_folded = vec_to_fold.into_iter().map(|res|{
            let new_res = alohomora::fold::fold(res).unwrap();
            new_res
        }).collect::<Vec<BBox<ApplicationResponseOut, AnyPolicy>>>();
        
        let vec_folded = alohomora::fold::fold(vec_folded).unwrap();
        times.push(t.elapsed());
        // println!("folded {}", fold);
        execute_pcr(vec_folded, PrivacyCriticalRegion::new(|vec_folded, _, _|{
            println!("can't optimize out the vec {:?}", vec_folded);
        },
        Signature{username: "hi", signature: "hi"},
        Signature{username: "hi", signature: "hi"},
        Signature{username: "hi", signature: "hi"},), ()).unwrap();
        
    }

    println!("folding: {:?}", compute_times(times));
}