// Database helpers: connecting, priming, and building the benchmark query.

use mysql::prelude::Queryable;
use mysql::Conn;

use sesame_mysql::{PConOpts, SesameConn};

pub const DB_NAME: &str = "schema_policy_bench";

// Column layout of `bench`, which is also the `SELECT *` order:
//   0: id        INT  (primary key)
//   1: owner     VARCHAR   <- pulled into the policy by ScorePolicy::from_row
//   2: category  INT       <- summed and revealed by the consume logic
//   3: score     INT       <- carries ScorePolicy (see policy.rs, column = 3)
//   4: flag      INT
//   5: label     VARCHAR
pub const SUM_COLUMN: usize = 2;

fn database_url() -> String {
    std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| String::from("mysql://root:password@127.0.0.1"))
}

fn bench_url() -> String {
    format!("{}/{}", database_url(), DB_NAME)
}

pub fn connect_sesame() -> SesameConn {
    SesameConn::new(PConOpts::from_url(&bench_url()).unwrap()).unwrap()
}

pub fn connect_plain() -> Conn {
    Conn::new(PConOpts::from_url(&bench_url()).unwrap()).unwrap()
}

// The benchmark query. Ordering by the primary key makes the returned rows
// (and therefore the summed column) deterministic and identical across
// variants, so their results can be cross-checked.
pub fn select(k: usize) -> String {
    format!("SELECT * FROM bench ORDER BY id LIMIT {}", k)
}

// Drop and recreate the throwaway database and populate `bench` with `rows`
// deterministic rows.
pub fn prime(rows: usize) {
    let mut conn = Conn::new(PConOpts::from_url(&database_url()).unwrap()).unwrap();
    conn.query_drop(format!("DROP DATABASE IF EXISTS {}", DB_NAME))
        .unwrap();
    conn.query_drop(format!("CREATE DATABASE {}", DB_NAME))
        .unwrap();
    conn.query_drop(format!("USE {}", DB_NAME)).unwrap();
    conn.query_drop(
        "CREATE TABLE bench (
            id       INT PRIMARY KEY AUTO_INCREMENT,
            owner    VARCHAR(64) NOT NULL,
            category INT NOT NULL,
            score    INT NOT NULL,
            flag     INT NOT NULL,
            label    VARCHAR(64) NOT NULL
        )",
    )
    .unwrap();

    let owners = ["alice", "bob", "carol", "admin"];
    let mut stmt = String::from("INSERT INTO bench (owner, category, score, flag, label) VALUES ");
    for i in 0..rows {
        if i > 0 {
            stmt.push(',');
        }
        stmt.push_str(&format!(
            "('{}', {}, {}, {}, 'row-{}')",
            owners[i % owners.len()],
            i % 7,
            (i * 3) % 100,
            i % 2,
            i
        ));
    }
    conn.query_drop(stmt).unwrap();
}
