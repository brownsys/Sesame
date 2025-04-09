extern crate alohomora;

use alohomora::bbox::BBox;
use alohomora::context::Context;
use alohomora::db::BBoxOpts;
use alohomora::k9db::db::BBoxK9dbRow;
use alohomora::k9db::K9db;
use alohomora::policy::NoPolicy;
use alohomora::testing::TestContextData;
use crate::alohomora::policy::Policy;

pub fn main() {
    let context = Context::test(TestContextData::new(()));

    // Initialize a K9db connection instance.
    let k9db = K9db::new(
        "src/schema.json",
        BBoxOpts::from_url("mysql://root:password@0.0.0.0:10001/").unwrap()
    ).unwrap();

    // Prime k9db (both DB schema and policy).
    k9db.prime().unwrap();

    let mut conn = k9db.make_connection().unwrap();
    conn.query_drop("CREATE VIEW lec_qcount as '\"SELECT questions.lecture_id, COUNT(questions.id) AS qcount FROM questions GROUP BY questions.lecture_id\"';").unwrap();
    conn.query_drop("CREATE VIEW agg_remote as '\"SELECT users.is_remote, AVG(answers.grade) as ucount FROM users JOIN answers on users.email = answers.author GROUP BY users.is_remote\"';").unwrap();
    // conn.query_drop("CREATE VIEW agg_gender as '\"SELECT users.gender, SUM(answers.grade) as ucount FROM users JOIN answers on users.email = answers.author GROUP BY users.gender\"';").unwrap();
    conn.query_drop("CREATE VIEW ml_training as '\"SELECT answers.grade, answers.submitted_at FROM answers JOIN users on answers.author = users.email WHERE users.consent_ml = 1\"';").unwrap();
    conn.query_drop("CREATE VIEW employers_release as '\"SELECT answers.author, AVG(answers.grade) FROM answers GROUP BY answers.author\"';").unwrap();
    conn.query_drop("CREATE VIEW total_avg as '\"SELECT AVG(grade) FROM answers\"';").unwrap();

    // Inert some data.
    let mut conn = k9db.make_connection().unwrap();
    conn.query_drop("INSERT INTO users VALUES ('admin@email.com', 'apikey1', 1, 0, 0, 0, 'M')").unwrap();
    conn.query_drop("INSERT INTO users VALUES ('student1@email.com', 'apikey2', 0, 0, 0, 0, 'M')").unwrap();
    conn.query_drop("INSERT INTO users VALUES ('student2@email.com', 'apikey3', 0, 1, 0, 0, 'F')").unwrap();
    conn.query_drop("INSERT INTO users VALUES ('student3@email.com', 'apikey3', 0, 1, 1, 1, 'F')").unwrap();
    conn.query_drop("INSERT INTO lectures(title) VALUES ('lecture 1')").unwrap();
    conn.query_drop("INSERT INTO lectures(title) VALUES ('lecture 2')").unwrap();
    conn.query_drop("INSERT INTO questions(lecture_id, question_number, question_text) VALUES (1, 1, 'Question 1')").unwrap();
    conn.query_drop("INSERT INTO questions(lecture_id, question_number, question_text) VALUES (2, 1, 'Question 2')").unwrap();
    conn.query_drop("INSERT INTO discussion_leaders(lecture_id, email) VALUES (1, 'student2@email.com')").unwrap();
    conn.query_drop("INSERT INTO discussion_leaders(lecture_id, email) VALUES (1, 'student3@email.com')").unwrap();
    conn.query_drop("INSERT INTO answers(question_id, lecture_id, author, answer, submitted_at, grade) VALUES (1, 1, 'student1@email.com', 'a1', '2020-01-01 00:00:00', 100)").unwrap();
    conn.query_drop("INSERT INTO answers(question_id, lecture_id, author, answer, submitted_at, grade) VALUES (1, 1, 'student2@email.com', 'a2', '2020-01-01 00:00:00', 50)").unwrap();
    conn.query_drop("INSERT INTO answers(question_id, lecture_id, author, answer, submitted_at, grade) VALUES (1, 1, 'student3@email.com', 'a3', '2020-01-01 00:00:00', 75)").unwrap();
    conn.query_drop("INSERT INTO answers(question_id, lecture_id, author, answer, submitted_at, grade) VALUES (2, 2, 'student1@email.com', 'a4', '2020-01-01 00:00:00', 80)").unwrap();
    conn.query_drop("INSERT INTO answers(question_id, lecture_id, author, answer, submitted_at, grade) VALUES (2, 2, 'student2@email.com', 'a5', '2020-01-01 00:00:00', 60)").unwrap();
    conn.query_drop("INSERT INTO answers(question_id, lecture_id, author, answer, submitted_at, grade) VALUES (2, 2, 'student3@email.com', 'a6', '2020-01-01 00:00:00', 10)").unwrap();
}
