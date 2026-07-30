// The schema policy under test.
//
// `ScorePolicy` is registered (via `#[schema_policy]`) on the `score` column of
// the `bench` table. Its constructor pulls the row's `owner` value into the
// policy struct, simulating a real policy whose decision depends on data
// carried alongside the protected cell. Registration is process-global and runs
// in a `ctor` before main, so this single declaration serves both Sesame
// variants in the benchmark (old and new).

use sesame::context::UnprotectedContext;
use sesame::policy::{Reason, SimplePolicy};
use sesame_mysql::{schema_policy, SchemaPolicy};

#[derive(Clone)]
#[schema_policy(table = "bench", column = 3)]
pub struct ScorePolicy {
    pub owner: String,
}

impl SimplePolicy for ScorePolicy {
    fn simple_name(&self) -> String {
        String::from("ScorePolicy")
    }
    fn simple_check(&self, context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
        match context.downcast_ref::<String>() {
            Some(user) => user == &self.owner || user == "admin",
            None => false,
        }
    }
    fn simple_join_direct(&mut self, other: &mut Self) {
        if self.owner != other.owner {
            self.owner = String::new();
        }
    }
}

impl SchemaPolicy for ScorePolicy {
    fn from_row(_table_name: &str, row: &mysql::Row) -> Self {
        ScorePolicy {
            owner: row.get(1).unwrap(),
        }
    }
}
