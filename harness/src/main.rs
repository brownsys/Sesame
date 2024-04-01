use alohomora::testing::BBoxClient;
use chrono::NaiveDateTime;
use fake::faker::company::en::Bs;
use fake::faker::internet::en::FreeEmail;
use fake::{Dummy, Fake, Faker};
use rocket::http::Cookie;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::iter::FromIterator;
use std::time::{Duration, Instant};
use websubmit_boxed::{make_rocket, parse_args};

const N_USERS: u32 = 10;
const N_LECTURES: u32 = 10;
const N_QUESTIONS_PER_LECTURE: u32 = 10;
const N_PREDICTION_ATTEMPTS_PER_LECTURE: u32 = 10;
const N_AGGREGATE_GRADES_QUERIES: u32 = 10;

const ADMIN_APIKEY: &'static str = "hashartem@brown.edu";

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

fn register_users(client: &BBoxClient, users: &mut Vec<User>) -> Vec<Duration> {
    users
        .iter_mut()
        .map(|user| {
            let request = client
                .post("/apikey/generate")
                .header(rocket::http::ContentType::Form)
                .body(serde_html_form::to_string(&user).unwrap());

            let now = Instant::now();
            request.dispatch();
            let elapsed = now.elapsed();

            user.token = format!("hash{}", user.email);

            elapsed
        })
        .collect()
}

fn add_lectures(client: &BBoxClient) -> Vec<Duration> {
    let mut lectures: Vec<Lecture> = (0..N_LECTURES).map(|_| Faker.fake()).collect();

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

fn add_questions(client: &BBoxClient) -> Vec<Duration> {
    (0..N_LECTURES)
        .map(|lecture_id| {
            let mut questions: Vec<Question> =
                (0..N_QUESTIONS_PER_LECTURE).map(|_| Faker.fake()).collect();

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

fn answer_questions(client: &BBoxClient, users: &Vec<User>) -> Vec<Duration> {
    users
        .iter()
        .map(|user| {
            (0..N_LECTURES)
                .map(|lecture_id| {
                    let answers: Vec<(String, String)> = (0..N_QUESTIONS_PER_LECTURE)
                        .map(|i| (format!("answers.{}", i), Faker.fake()))
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

fn view_answers(client: &BBoxClient) -> Vec<Duration> {
    (0..N_LECTURES)
        .map(|lecture_id| {
            let request = client
                .get(format!("/answers/{}", lecture_id))
                .cookie(Cookie::new("apikey", ADMIN_APIKEY));

            let now = Instant::now();
            request.dispatch();
            let elapsed = now.elapsed();

            elapsed
        })
        .collect()
}

fn submit_grades(client: &BBoxClient, users: &Vec<User>) -> Vec<Duration> {
    users
        .iter()
        .map(|user| {
            (0..N_LECTURES)
                .map(|lecture_id| {
                    (0..N_QUESTIONS_PER_LECTURE)
                        .map(|question_id| {
                            let grade: Grade = Faker.fake();

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

fn predict_grades(client: &BBoxClient) -> Vec<Duration> {
    (0..N_LECTURES)
        .map(|lecture_id| {
            (0..N_PREDICTION_ATTEMPTS_PER_LECTURE)
                .map(|_| {
                    let timestamp: NaiveDateTime = Faker.fake();
                    let prediction_request = PredictionRequest {
                        time: timestamp.format("%Y-%m-%d %H:%M:%S").to_string(),
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
        })
        .flatten()
        .collect()
}

fn get_aggregates(client: &BBoxClient) -> Vec<Duration> {
    (0..N_AGGREGATE_GRADES_QUERIES)
        .map(|_| {
            let request = client
                .get("/manage/users")
                .cookie(Cookie::new("apikey", ADMIN_APIKEY));

            let now = Instant::now();
            request.dispatch();
            let elapsed = now.elapsed();

            elapsed
        })
        .collect()
}

fn write_stats(name: &'static str, data: &Vec<Duration>) {
    let mut sorted_data = data.to_owned();
    sorted_data.sort();

    fs::write(
        name,
        format!(
            "{}\n50-th percentile: {:?}\n95-th percentile: {:?}\n99-th percentile: {:?}\n",
            name,
            sorted_data.get((sorted_data.len() as f32 * 0.50).floor() as usize),
            sorted_data.get((sorted_data.len() as f32 * 0.95).floor() as usize),
            sorted_data.get((sorted_data.len() as f32 * 0.99).floor() as usize),
        ),
    ).unwrap();
}

fn main() {
    let args = parse_args();
    let rocket = make_rocket(args);
    let client = BBoxClient::tracked(rocket).expect("valid `Rocket`");

    let mut users: Vec<User> = (0..N_USERS).map(|_| Faker.fake()).collect();

    // 1. Bench generating ApiKeys.
    let register_users_bench = register_users(&client, &mut users);
    write_stats("register_users_bench", &register_users_bench);

    // Prime the database with other data.
    add_lectures(&client);
    add_questions(&client);

    // 2. Bench answering the questions.
    let answer_questions_bench = answer_questions(&client, &users);
    write_stats("answer_questions_bench", &answer_questions_bench);

    // 3. Bench viewing answers for a lecture.
    let view_answers_bench = view_answers(&client);
    write_stats("view_answers_bench", &view_answers_bench);

    // 4. Submit a grade and retrain the prediction model.
    let submit_grades_bench = submit_grades(&client, &users);
    write_stats("submit_grades_bench", &submit_grades_bench);

    // 5. Query the prediction model.
    let predict_grades_bench = predict_grades(&client);
    write_stats("predict_grades_bench", &predict_grades_bench);

    // 6. Query aggregate generation.
    // let get_aggregates_bench = get_aggregates(&client);
    // write_stats("get_aggregates_bench", &get_aggregates_bench);
}
