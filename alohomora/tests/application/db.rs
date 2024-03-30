use alohomora::bbox::BBox;
use alohomora::db::{BBoxConn, BBoxOpts, BBoxResult, from_value};
use alohomora::policy::Policy;
use crate::application::context::AppContext;
use crate::application::models::Grade;

pub struct DB {
    conn: BBoxConn,
}

impl DB {
    pub fn connect() -> DB {
        let opts = BBoxOpts::from_url("mysql://root:password@127.0.0.1/").unwrap();
        DB { conn: BBoxConn::new(opts).unwrap() }
    }

    pub fn prime(&mut self) {
        for stmt in include_str!("schema.sql").split(";") {
            if stmt.trim().len() > 0 {
                self.conn.query_drop(stmt).unwrap();
            }
        }
    }

    pub fn read_by_user<P: Policy + Clone + 'static>(&mut self, user: BBox<String, P>, context: AppContext) -> Vec<Grade> {
        let result = self.conn.prep_exec_iter(
            "SELECT * FROM grades WHERE name = ?",
            (user, ),
            context,
        ).unwrap();

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
        let result = self.conn.prep_exec_iter(
            "SELECT * FROM grades",
            (),
            context,
        ).unwrap();

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

    pub fn insert<P1: Policy + Clone + 'static, P2: Policy + Clone + 'static>(
        &mut self, user: BBox<String, P1>,
        grade: BBox<u64, P2>,
        context: AppContext
    ) -> BBoxResult<()> {
        self.conn.prep_exec_drop(
            "INSERT INTO grades(name, grade) VALUES (?, ?)",
            (user, grade),
            context
        )
    }
}