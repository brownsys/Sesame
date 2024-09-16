use chrono::NaiveDateTime;
use fake::faker::company::en::Bs;
use fake::faker::internet::en::FreeEmail;
use fake::{Dummy, Fake, Faker};
use rocket::http::Cookie;

use serde::Serialize;

use std::fs;
use std::time::{Duration, Instant};

use alohomora::testing::BBoxClient;
use rocket::local::blocking::Client;

use rand::rngs::StdRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

use websubmit::{make_rocket as ws_make_rocket, parse_args as ws_parse_args};
use websubmit_boxed::{make_rocket as wsb_make_rocket, parse_args as wsb_parse_args};

const RNG_SEED: u64 = 3705;

const N_USERS: usize = 100;
const N_LECTURES: usize = 10;
const N_QUESTIONS_PER_LECTURE: usize = 10;

const N_REGISTRATION_ATTEMPTS: usize = 1000;

const N_ANSWER_VIEW_ATTEMPTS_PER_LECTURE: usize = 100;
const PREDICTION_REQUEST_BATCH_SIZE: usize = 100;
const N_PREDICTION_ATTEMPTS_PER_LECTURE: usize = 100;
const N_DISCUSSION_LEADER_QUERIES_PER_LECTURE: usize = 100;

const N_RETRAINING_MODEL_QUERIES: usize = 1000;
const N_AGGREGATE_GRADES_QUERIES: usize = 1000;
const N_EMPLOYER_INFO_QUERIES: usize = 1000;

const ADMIN_APIKEY: &'static str = "ADMIN_API_KEY";
const DISCUSSION_LEADER_KEY: &'static str = "DISCUSSION_LEADER_KEY";

#[derive(Debug, Dummy, Serialize)]
enum Gender {
    M,
    F,
    X,
}

// I know race and ethnicity are different -- I just did not want to define an enum with 1500+ fields.
#[derive(Debug, Dummy, Serialize)]
enum Ethnicity {
    White,
    Black,
    TwoOrMore,
    Other,
    Asian,
    NativeAmerican,
    NativePacific,
}

#[derive(Debug, Dummy, Serialize)]
enum Education {
    HighSchool,
    CommunityCollege,
    Bachelors,
    Masters,
    PhD,
}

#[derive(Debug, Dummy, Serialize)]
struct User {
    #[dummy(faker = "FreeEmail()")]
    email: String,
    #[dummy(faker = "17..45")]
    age: u32,

    gender: Gender,
    ethnicity: Ethnicity,
    education: Education,

    is_remote: bool,
    consent: bool,

    #[dummy(default)]
    token: String,
}

#[derive(Debug, Dummy, Serialize)]
struct Lecture {
    #[dummy(default)]
    lec_id: u8,
    #[dummy(faker = "Bs()")]
    lec_label: String,
}

#[derive(Debug, Dummy, Serialize)]
struct Question {
    #[dummy(default)]
    q_id: u64,
    #[dummy(faker = "Bs()")]
    q_prompt: String,
}

#[derive(Debug, Dummy, Serialize)]
struct Grade {
    #[dummy(faker = "0..=100")]
    grade: u64,
}

#[derive(Debug, Serialize)]
struct PredictionRequest {
    time: String,
}

fn register_users(client: &Client, users: &mut Vec<User>) -> Vec<Duration> {
    users
        .iter_mut()
        .map(|user| {
            let request = client
                .post("/apikey/generate")
                .header(rocket::http::ContentType::Form)
                .body(serde_html_form::to_string(&user).unwrap());

            let now = Instant::now();
            let response = request.dispatch();
            let elapsed = now.elapsed();

            // println!("response {:?}", response);

            let json: serde_json::Value = response.into_json().unwrap();
            let apikey: serde_json::Value = json.get("apikey").unwrap().to_owned();

            user.token = apikey.as_str().unwrap().to_owned();

            elapsed
        })
        .collect()
}

fn add_lectures(client: &Client, r: &mut StdRng) -> Vec<Duration> {
    let mut lectures: Vec<Lecture> = (0..N_LECTURES).map(|_| Faker.fake_with_rng(r)).collect();

    lectures
        .iter_mut()
        .enumerate()
        .map(|(i, lecture)| {
            lecture.lec_id = i as u8;

            let request = client
                .post("/admin/lec/add")
                .cookie(Cookie::new("apikey", ADMIN_APIKEY))
                .header(rocket::http::ContentType::Form)
                .body(serde_html_form::to_string(&lecture).unwrap());

            let now = Instant::now();
            request.dispatch();
            let elapsed = now.elapsed();

            elapsed
        })
        .collect()
}

fn add_questions(client: &Client, r: &mut StdRng) -> Vec<Duration> {
    (0..N_LECTURES)
        .map(|lecture_id| {
            let mut questions: Vec<Question> = (0..N_QUESTIONS_PER_LECTURE)
                .map(|_| Faker.fake_with_rng(r))
                .collect();

            questions
                .iter_mut()
                .enumerate()
                .map(|(i, question)| {
                    question.q_id = i as u64;

                    let request = client
                        .post(format!("/admin/lec/{}", lecture_id))
                        .cookie(Cookie::new("apikey", ADMIN_APIKEY))
                        .header(rocket::http::ContentType::Form)
                        .body(serde_html_form::to_string(&question).unwrap());

                    let now = Instant::now();
                    request.dispatch();
                    let elapsed = now.elapsed();

                    elapsed
                })
                .collect::<Vec<Duration>>()
        })
        .flatten()
        .collect()
}

fn answer_questions(client: &Client, users: &Vec<User>, r: &mut StdRng) -> Vec<Duration> {
    users
        .iter()
        .map(|user| {
            (0..N_LECTURES)
                .map(|lecture_id| {
                    let answers: Vec<(String, String)> = (0..N_QUESTIONS_PER_LECTURE)
                        .map(|i| (format!("answers.{}", i), Faker.fake_with_rng(r)))
                        .collect();

                    let request = client
                        .post(format!("/questions/{}", lecture_id))
                        .cookie(Cookie::new("apikey", user.token.as_str()))
                        .header(rocket::http::ContentType::Form)
                        .body(serde_html_form::to_string(&answers).unwrap());

                    let now = Instant::now();
                    request.dispatch();
                    let elapsed = now.elapsed();

                    elapsed
                })
                .collect::<Vec<Duration>>()
        })
        .flatten()
        .collect()
}

fn view_answers(client: &Client, r: &mut StdRng) -> Vec<Duration> {
    let mut samples: Vec<usize> = (0..N_LECTURES)
        .cycle()
        .take(N_LECTURES * N_ANSWER_VIEW_ATTEMPTS_PER_LECTURE)
        .collect();
    samples.shuffle(r);

    samples
        .iter()
        .map(|lecture_id| {
            let request = client
                .get(format!("/answers/{}", lecture_id))
                .cookie(Cookie::new("apikey", ADMIN_APIKEY));

            let now = Instant::now();
            request.dispatch();
            let elapsed = now.elapsed();

            elapsed
        })
        .collect::<Vec<Duration>>()
}

fn view_answers_naive(client: &Client, r: &mut StdRng) -> Vec<Duration> {
    let mut samples: Vec<usize> = (0..N_LECTURES)
        .cycle()
        .take(N_LECTURES * N_ANSWER_VIEW_ATTEMPTS_PER_LECTURE)
        .collect();
    samples.shuffle(r);

    samples
        .iter()
        .map(|lecture_id| {
            let request = client
                .get(format!("/answers/naive/{}", lecture_id))
                .cookie(Cookie::new("apikey", ADMIN_APIKEY));

            let now = Instant::now();
            request.dispatch();
            let elapsed = now.elapsed();

            elapsed
        })
        .collect::<Vec<Duration>>()
}

fn submit_grades(client: &Client, users: &Vec<User>, r: &mut StdRng) -> Vec<Duration> {
    users
        .iter()
        .map(|user| {
            (0..N_LECTURES)
                .map(|lecture_id| {
                    (0..N_QUESTIONS_PER_LECTURE)
                        .map(|question_id| {
                            let grade: Grade = Faker.fake_with_rng(r);

                            let request = client
                                .post(format!(
                                    "/grades/editg/{}/{}/{}",
                                    user.email, lecture_id, question_id
                                ))
                                .cookie(Cookie::new("apikey", ADMIN_APIKEY))
                                .header(rocket::http::ContentType::Form)
                                .body(serde_html_form::to_string(&grade).unwrap());

                            let now = Instant::now();
                            request.dispatch();
                            let elapsed = now.elapsed();

                            elapsed
                        })
                        .collect::<Vec<Duration>>()
                })
                .flatten()
                .collect::<Vec<Duration>>()
        })
        .flatten()
        .collect()
}

fn predict_grades(client: &Client, r: &mut StdRng) -> Vec<Duration> {
    let mut samples: Vec<usize> = (0..N_LECTURES)
        .cycle()
        .take(N_LECTURES * N_PREDICTION_ATTEMPTS_PER_LECTURE)
        .collect();
    samples.shuffle(r);

    samples
        .iter()
        .map(|lecture_id| {
            let prediction_request = PredictionRequest {
                time: (0..PREDICTION_REQUEST_BATCH_SIZE)
                    .map(|_| {
                        let timestamp: NaiveDateTime = Faker.fake_with_rng(r);
                        timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
                    })
                    .collect::<Vec<_>>()
                    .join(","),
            };

            let request = client
                .post(format!("/predict/predict_grade/{}", lecture_id))
                .cookie(Cookie::new("apikey", ADMIN_APIKEY))
                .header(rocket::http::ContentType::Form)
                .body(serde_html_form::to_string(&prediction_request).unwrap());

            let now = Instant::now();
            request.dispatch();
            let elapsed = now.elapsed();

            elapsed
        })
        .collect::<Vec<Duration>>()
}

fn retrain_model(client: &Client) -> Vec<Duration> {
    (0..N_RETRAINING_MODEL_QUERIES)
        .map(|_| {
            let request = client
                .get("/predict/retrain_model")
                .cookie(Cookie::new("apikey", ADMIN_APIKEY));

            let now = Instant::now();
            request.dispatch();
            let elapsed = now.elapsed();

            elapsed
        })
        .collect()
}

fn get_aggregates(client: &Client) -> Vec<Duration> {
    (0..N_AGGREGATE_GRADES_QUERIES)
        .map(|_| {
            let request = client
                .get("/manage/remote")
                .cookie(Cookie::new("apikey", ADMIN_APIKEY));

            let now = Instant::now();
            request.dispatch();
            let elapsed = now.elapsed();

            elapsed
        })
        .collect()
}

fn get_employer_info(client: &Client) -> Vec<Duration> {
    (0..N_EMPLOYER_INFO_QUERIES)
        .map(|_| {
            let request = client
                .get("/manage/employers")
                .cookie(Cookie::new("apikey", ADMIN_APIKEY));

            let now = Instant::now();
            request.dispatch();
            let elapsed = now.elapsed();

            elapsed
        })
        .collect()
}

fn get_discussion_leader(client: &Client, r: &mut StdRng) -> Vec<Duration> {
    let mut samples: Vec<usize> = (0..N_LECTURES)
        .cycle()
        .take(N_LECTURES * N_DISCUSSION_LEADER_QUERIES_PER_LECTURE)
        .collect();
    samples.shuffle(r);

    samples
        .iter()
        .map(|lecture_id| {
            let request = client
                .get(format!("/answers/discussion_leaders/{}", lecture_id))
                .cookie(Cookie::new("apikey", DISCUSSION_LEADER_KEY));

            let now = Instant::now();
            request.dispatch();
            let elapsed = now.elapsed();

            elapsed
        })
        .collect::<Vec<Duration>>()
}

fn get_discussion_leader_naive(client: &Client, r: &mut StdRng) -> Vec<Duration> {
    let mut samples: Vec<usize> = (0..N_LECTURES)
        .cycle()
        .take(N_LECTURES * N_DISCUSSION_LEADER_QUERIES_PER_LECTURE)
        .collect();
    samples.shuffle(r);

    samples
        .iter()
        .map(|lecture_id| {
            let request = client
                .get(format!("/answers/discussion_leaders/naive/{}", lecture_id))
                .cookie(Cookie::new("apikey", DISCUSSION_LEADER_KEY));

            let now = Instant::now();
            request.dispatch();
            let elapsed = now.elapsed();

            elapsed
        })
        .collect::<Vec<Duration>>()
}

fn write_stats(name: String, data: &Vec<Duration>) {
    let duration_nanos: Vec<u128> = data.iter().map(|d| d.as_nanos()).collect();
    fs::create_dir_all("benches/").unwrap();
    fs::write(
        format!("benches/{}.json", name),
        serde_json::to_string_pretty(&duration_nanos).unwrap(),
    )
    .unwrap();
}

#[cfg(feature = "boxed")]
fn get_websubmit() -> BBoxClient {
    println!("Running boxed websubmit.");
    BBoxClient::tracked(wsb_make_rocket(wsb_parse_args())).expect("valid `Rocket`")
}

#[cfg(feature = "unboxed")]
fn get_websubmit() -> Client {
    println!("Running regular websubmit.");
    Client::tracked(ws_make_rocket(ws_parse_args())).expect("valid `Rocket`")
}

fn main() {
    let ref mut r = StdRng::seed_from_u64(RNG_SEED);

    let client = get_websubmit();
    let used_client: &Client = &client;

    let prefix = if cfg!(feature = "boxed") {
        "boxed_".to_owned()
    } else if cfg!(feature = "unboxed") {
        "".to_owned()
    } else {
        unreachable!()
    };

    let mut users: Vec<User> = (0..N_REGISTRATION_ATTEMPTS)
        .map(|_| Faker.fake_with_rng(r))
        .collect();

    // 1. Bench generating ApiKeys.
    let register_users_bench = register_users(&used_client, &mut users);
    write_stats(
        prefix.clone() + "register_users_bench",
        &register_users_bench,
    );
    println!("Created {} user accounts.", users.len());

    users = users.into_iter().take(N_USERS as usize).collect();
    println!("Using only {} users to benchmark.", users.len());

    // Prime the database with other data.
    add_lectures(&used_client, r);
    add_questions(&used_client, r);

    // 2. Bench answering the questions.
    let answer_questions_bench = answer_questions(&used_client, &users, r);
    write_stats(
        prefix.clone() + "answer_questions_bench",
        &answer_questions_bench,
    );
    println!(
        "Took {} samples for answer questions endpoint.",
        answer_questions_bench.len()
    );

    // 3. Bench viewing answers for a lecture.
    let view_answers_bench = view_answers(&used_client, r);
    write_stats(prefix.clone() + "view_answers_bench", &view_answers_bench);
    println!(
        "Took {} samples for view answers endpoint.",
        view_answers_bench.len()
    );

    // Prime the database with grades.
    submit_grades(&used_client, &users, r);

    // 4. Bench retraining the model.
    let retrain_model_bench = retrain_model(&used_client);
    write_stats(prefix.clone() + "retrain_model_bench", &retrain_model_bench);
    println!(
        "Took {} samples for retrain model endpoint.",
        retrain_model_bench.len()
    );

    // 5. Query the prediction model.
    let predict_grades_bench = predict_grades(&used_client, r);
    write_stats(
        prefix.clone() + "predict_grades_bench",
        &predict_grades_bench,
    );
    println!(
        "Took {} samples for predict grades endpoint.",
        predict_grades_bench.len()
    );

    // 6. Query aggregate generation.
    let get_aggregates_bench = get_aggregates(&used_client);
    write_stats(
        prefix.clone() + "get_aggregates_bench",
        &get_aggregates_bench,
    );
    println!(
        "Took {} samples for get aggregates endpoint.",
        get_aggregates_bench.len()
    );

    // 7. Employer info generation.
    let get_employer_info_bench = get_employer_info(&used_client);
    write_stats(
        prefix.clone() + "get_employer_info_bench",
        &get_employer_info_bench,
    );
    println!(
        "Took {} samples for get employer info endpoint.",
        get_employer_info_bench.len()
    );

    // FOLD EXPERIMENT.

    // Discussion leader normal (runs for both boxed and unboxed).
    let get_discussion_leader_bench = get_discussion_leader(&used_client, r);
    write_stats(
        prefix.clone() + "get_discussion_leader_bench",
        &get_discussion_leader_bench,
    );
    println!(
        "Took {} samples for get discussion leader endpoint.",
        get_discussion_leader_bench.len()
    );

    if cfg!(feature = "boxed") {
        let get_discussion_leader_naive_bench = get_discussion_leader_naive(&used_client, r);
        write_stats(
            prefix.clone() + "get_discussion_leader_naive_bench",
            &get_discussion_leader_naive_bench,
        );
        println!(
            "Took {} samples for get discussion leader naive endpoint.",
            get_discussion_leader_naive_bench.len()
        );

        let view_answers_naive_bench = view_answers_naive(&used_client, r);
        write_stats(
            prefix.clone() + "view_answers_naive_bench",
            &view_answers_naive_bench,
        );
        println!(
            "Took {} samples for view answers naive endpoint.",
            view_answers_naive_bench.len()
        );
    }
}
