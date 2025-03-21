use std::sync::{Arc, Mutex};
use std::time::{Instant, Duration};

use alohomora::testing::BBoxClient;
use rocket::{http::{Cookie, Header, Status}, local::blocking::Client};

// Define all portfolio crates conditionally on features.
#[cfg(feature = "boxed")]
extern crate portfolio_boxed_api;
#[cfg(feature = "boxed")]
extern crate portfolio_boxed_core;
#[cfg(feature = "unboxed")]
extern crate portfolio_api;
#[cfg(feature = "unboxed")]
extern crate portfolio_core;

// Use APIs and types from portfolio conditionally on features.
#[cfg(feature = "boxed")]
use portfolio_boxed_api::*;
#[cfg(feature = "unboxed")]
use portfolio_api::*;

// Rename APIs in portfolio_core to the boxed version.
#[cfg(feature = "boxed")]
use portfolio_boxed_core::models::{application::CleanApplicationResponse, candidate::CleanCreateCandidateResponse};
#[cfg(feature = "unboxed")]
use portfolio_core::models::{application::ApplicationResponse as CleanApplicationResponse, candidate::CreateCandidateResponse as CleanCreateCandidateResponse};

// Create benchmarking rocket client conditionally on feature.
#[cfg(feature = "boxed")]
fn get_portfolio() -> BBoxClient {
    BBoxClient::tracked(rocket()).expect("invalid rocket")
}
#[cfg(feature = "unboxed")]
fn get_portfolio() -> Client {
    Client::tracked(rocket()).expect("invalid rocket")
}

// No more conditionals!
pub const ADMIN_ID: i32 = 3;
pub const ADMIN_PASSWORD: &'static str = "test";

// Helpers for login.
pub fn admin_login(client: &Client) -> (Cookie, Cookie) {
    let response = client
        .post("/admin/login")
        .body(format!(
            "{{
        \"adminId\": {},
        \"password\": \"{}\"
    }}",
            ADMIN_ID, ADMIN_PASSWORD
        ))
        .dispatch();
    assert_eq!(response.status(), Status::Ok);
    (response.cookies().get("id").unwrap().to_owned(), response.cookies().get("key").unwrap().to_owned())
}

pub fn candidate_login(client: &Client, id: i32, password: String) -> (Cookie, Cookie) {
    let response = client
        .post("/candidate/login")
        .body(format!(
            "{{
        \"applicationId\": {},
        \"password\": \"{}\"
    }}",
            id, password
        ))
        .dispatch();

    (
        response.cookies().get("id").unwrap().to_owned(),
        response.cookies().get("key").unwrap().to_owned(),
    )
}

// Helpers for creating candidates/users.
fn create_candidate(
    client: &Client,
    cookies: (Cookie, Cookie),
    id: i32,
    pid: String,
) -> CleanCreateCandidateResponse {
    let response = client
        .post("/admin/create")
        .body(format!(
            "{{
        \"applicationId\": {},
        \"personalIdNumber\": \"{}\"
    }}",
            id, pid
        ))
        .cookie(cookies.0)
        .cookie(cookies.1)
        .dispatch();

    assert_eq!(response.status(), Status::Ok);

    response.into_json::<CleanCreateCandidateResponse>().unwrap()
}

fn make_candidates(client: &Client, ids: Vec<i32>) -> Vec<(i32, String)> {
    let mut cands = Vec::new();
    let cookies = admin_login(&client);

    for id in ids {
        let personal_id = id % 1000;
        let response = create_candidate(&client, cookies.clone(), id, personal_id.to_string());
        cands.push((id, response.password));
        println!("{}", cands.len());
    }
    cands
}

// Listing candidates paginated 50 at a time.
fn list_candidates(
    times_to_list: u64,
    client: &Client,
    response_len: usize,
) -> Vec<Duration> {
    let mut times = vec![];
    let cookies = admin_login(&client);
    for i in 0..times_to_list {
        let request = client
            .get(format!("/admin/list/candidates?page={}", i % 50))
            .cookie(cookies.clone().0)
            .cookie(cookies.clone().1);

        let timer = Instant::now();
        let response = request.dispatch();

        assert_eq!(response.status(), Status::Ok);
        times.push(timer.elapsed());

        let vec = response.into_json::<Vec<CleanApplicationResponse>>().unwrap();
        assert_eq!(vec.len(), 20);
    }
    times
}


// Updated candidate details.
pub const CANDIDATE_DETAILS: &'static str = "{
    \"candidate\": {
        \"name\": \"idk\",
        \"surname\": \"idk\",
        \"birthSurname\": \"surname\",
        \"birthplace\": \"Praha 1\",
        \"birthdate\": \"2015-09-18\",
        \"address\": \"Stefanikova jidelna\",
        \"letterAddress\": \"Stefanikova jidelna\",
        \"telephone\": \"000111222333\",
        \"citizenship\": \"Czech Republic\",
        \"email\": \"magor@magor.cz\",
        \"sex\": \"MALE\",
        \"personalIdNumber\": \"0101010000\",
        \"schoolName\": \"29988383\",
        \"healthInsurance\": \"000\",
        \"grades\": [],
        \"firstSchool\": {\"name\": \"SSPŠ\", \"field\": \"KB\"},
        \"secondSchool\": {\"name\": \"SSPŠ\", \"field\": \"IT\"},
        \"testLanguage\": \"CZ\"
    },
    \"parents\": [
        {
            \"name\": \"maminka\",
            \"surname\": \"chad\",
            \"telephone\": \"420111222333\",
            \"email\": \"maminka@centrum.cz\"
        }
    ]
}";

fn upload_details(client: &Client, cands: Vec<(i32, String)>) -> Vec<Duration> {
    let mut times = vec![];
    for (id, password) in cands {
        // login
        let cookies = candidate_login(&client, id, password);
        let request = client
            .post("/candidate/details")
            .cookie(cookies.0.clone())
            .cookie(cookies.1.clone())
            .body(CANDIDATE_DETAILS.to_string());

        let timer = Instant::now();
        let response = request.dispatch();
        times.push(timer.elapsed());
        println!("{:?}", id);
        assert_eq!(response.status(), Status::Ok);
    }
    times
}

// Helper for finding statistics about runtime
fn compute_times(mut times: Vec<Duration>) -> (u64, u64, u64) {
    times.sort();
    let median = times[times.len() / 2].as_micros() as u64;
    let ninty = times[times.len() * 95 / 100].as_micros() as u64;
    let avg = times.iter().map(|t| t.as_micros() as u64).sum::<u64>() / times.len() as u64;
    (median, ninty, avg)
}

fn main() {
    // setup
    let client = get_portfolio();

    let ids: Vec<i32> = (102151..(102151 + 1000)).collect();
    let ids_len = ids.len();

    println!("making candidates");
    let candidates = make_candidates(&client, ids);
    println!("done making candidates");

    let upload_times = upload_details(&client, candidates);
    println!("details: {:?}", compute_times(upload_times));

    let list_times = list_candidates(100, &client, ids_len + 1);
    println!("list: {:?}", compute_times(list_times));
}
