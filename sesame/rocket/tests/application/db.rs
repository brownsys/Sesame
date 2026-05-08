use crate::application::context::AppContext;
use crate::application::models::Grade;

use sesame::pcon::PCon;
use sesame::policy::AnyPolicyable;

use sesame_mysql::{from_value, PConOpts, PConResult, SesameConn};

pub struct DB {
    conn: SesameConn,
}

impl DB {
    pub fn connect() -> DB {
        let opts = PConOpts::from_url("mysql://root:password@127.0.0.1/").unwrap();
        DB {
            conn: SesameConn::new(opts).unwrap(),
        }
    }

    pub fn prime(&mut self) {
        for stmt in include_str!("schema.sql").split(";") {
            if stmt.trim().len() > 0 {
                self.conn.query_drop(stmt).unwrap();
            }
        }
    }

    pub fn read_by_user<P: AnyPolicyable>(
        &mut self,
        user: PCon<String, P>,
        context: AppContext,
    ) -> Vec<Grade> {
        let result = self
            .conn
            .prep_exec_iter("SELECT * FROM grades WHERE name = ?", (user,), context)
            .unwrap();

        let result = result.map(|row| {
            let row = row.unwrap();
            Grade {
                id: from_value(row.get(0).unwrap()).unwrap(),
                name: from_value(row.get(1).unwrap()).unwrap(),
                grade: from_value(row.get(2).unwrap()).unwrap(),
            }
        });

        result.collect()
    }

    pub fn read_all(&mut self, context: AppContext) -> Vec<Grade> {
        let result = self
            .conn
            .prep_exec_iter("SELECT * FROM grades", (), context)
            .unwrap();

        let result = result.map(|row| {
            let row = row.unwrap();
            Grade {
                id: from_value(row.get(0).unwrap()).unwrap(),
                name: from_value(row.get(1).unwrap()).unwrap(),
                grade: from_value(row.get(2).unwrap()).unwrap(),
            }
        });

        result.collect()
    }

    pub fn insert<P1: AnyPolicyable, P2: AnyPolicyable>(
        &mut self,
        user: PCon<String, P1>,
        grade: PCon<u64, P2>,
        context: AppContext,
    ) -> PConResult<()> {
        self.conn.prep_exec_drop(
            "INSERT INTO grades(name, grade) VALUES (?, ?)",
            (user, grade),
            context,
        )
    }
}
