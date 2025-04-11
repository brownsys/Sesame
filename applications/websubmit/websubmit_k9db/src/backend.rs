use crate::policies::ContextData;
use alohomora::db::{BBoxOpts, BBoxParams, BBoxStatement, BBoxValue};
use slog::{debug, o, warn};
use std::collections::HashMap;
use std::error::Error;
use std::result::Result;
use alohomora::k9db::{K9db, db::K9dbBBoxConn};

use crate::policies::Context;

pub struct MySqlBackend {
    k9db: K9db,
    pub handle: K9dbBBoxConn,
    pub log: slog::Logger,
    prep_stmts: HashMap<String, BBoxStatement>,
    db_user: String,
    db_password: String,
    db_name: String,
}

impl MySqlBackend {
    pub fn new(
        user: &str,
        password: &str,
        db_port: &str,
        dbname: &str,
        log: Option<slog::Logger>,
        prime: bool,
    ) -> Result<Self, Box<dyn Error>> {
        let actual_port = if db_port == "" { db_port.to_owned() } else { format!(":{}", db_port) };
        let log = match log {
            None => slog::Logger::root(slog::Discard, o!()),
            Some(l) => l,
        };

        debug!(
            log,
            "Connecting to MySql DB and initializing schema {}...", dbname
        );

        let k9db = K9db::new(
            "src/schema.json",
            BBoxOpts::from_url("mysql://root:password@0.0.0.0:10001/").unwrap()
        ).unwrap();

        if prime {
            k9db.prime().unwrap();
        }

        let mut db = k9db.make_connection().unwrap();
        assert_eq!(db.ping(), true);

        if prime {
            let schema = std::fs::read_to_string("src/k9db.sql")?;
            for line in schema.lines() {
                if line.starts_with("--") || line.is_empty() {
                    continue;
                }
                db.query_drop(line).unwrap();
            }
        }

        Ok(MySqlBackend {
            k9db: k9db,
            handle: db,
            log: log,
            prep_stmts: HashMap::new(),
            db_user: String::from(user),
            db_password: String::from(password),
            db_name: String::from(dbname),
        })
    }

    fn reconnect(&mut self) {
        self.handle = self.k9db.make_connection().unwrap();
    }

    pub fn prep_exec<P: Into<BBoxParams>>(
        &mut self,
        sql: &str,
        params: P,
        context: Context,
    ) -> Vec<Vec<BBoxValue>> {
        if !self.prep_stmts.contains_key(sql) {
            let stmt = self
                .handle
                .prep(sql)
                .expect(&format!("failed to prepare statement \'{}\'", sql));
            self.prep_stmts.insert(sql.to_owned(), stmt);
        }

        let params: BBoxParams = params.into();
        loop {
            match self.handle.exec_iter(
                self.prep_stmts[sql].clone(),
                params.clone(),
                context.clone(),
            ) {
                Err(e) => {
                    warn!(
                        self.log,
                        "query \'{}\' failed ({}), reconnecting to database", sql, e
                    );
                }
                Ok(res) => {
                    let mut rows = vec![];
                    for row in res {
                        rows.push(row.unwrap().unwrap());
                    }
                    //debug!(self.log, "executed query {}, got {} rows", sql, rows.len());
                    return rows;
                }
            }
            self.reconnect();
        }
    }

    fn do_insert<P: Into<BBoxParams>>(
        &mut self,
        table: &str,
        vals: P,
        replace: bool,
        context: Context,
    ) {
        let vals: BBoxParams = vals.into();
        let mut param_count = 0;
        if let BBoxParams::Positional(vec) = &vals {
            param_count = vec.len();
        }

        let op = if replace { "REPLACE" } else { "INSERT" };
        let q = format!(
            "{} INTO {} VALUES ({})",
            op,
            table,
            (0..param_count)
                .map(|_| "?")
                .collect::<Vec<&str>>()
                .join(",")
        );
        while let Err(e) = self
            .handle
            .exec_drop(q.clone(), vals.clone(), context.clone())
        {
            warn!(
                self.log,
                "failed to insert into {}, query {} ({}), reconnecting to database", table, q, e
            );
            self.reconnect();
        }
    }

    pub fn insert<P: Into<BBoxParams>>(
        &mut self,
        table: &str,
        vals: P,
        context: Context,
    ) {
        self.do_insert(table, vals, false, context);
    }

    pub fn replace<P: Into<BBoxParams>>(
        &mut self,
        table: &str,
        vals: P,
        context: Context,
    ) {
        self.do_insert(table, vals, true, context);
    }
}
