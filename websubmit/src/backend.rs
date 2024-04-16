use mysql::prelude::*;
use mysql::Opts;
use mysql::*;
use std::collections::HashMap;

pub use mysql::Value;

use slog::{o, warn};

pub struct MySqlBackend {
    pub handle: mysql::Conn,
    pub log: slog::Logger,
    _schema: String,
    prep_stmts: HashMap<String, mysql::Statement>,
    db_user: String,
    db_password: String,
    db_name: String,
}

impl MySqlBackend {
    pub fn new(
        user: &str,
        password: &str,
        dbname: &str,
        log: Option<slog::Logger>,
        prime: bool,
    ) -> Result<Self> {
        let log = match log {
            None => slog::Logger::root(slog::Discard, o!()),
            Some(l) => l,
        };

        let schema = std::fs::read_to_string("src/schema.sql")?;

        // debug!(
        //     log,
        //     "Connecting to MySql DB and initializing schema {}...", dbname
        // );
        let mut db = mysql::Conn::new(
            Opts::from_url(&format!("mysql://{}:{}@127.0.0.1/", user, password)).unwrap(),
        )
        .unwrap();
        assert_eq!(db.ping(), true);

        if prime {
            db.query_drop(format!("DROP DATABASE IF EXISTS {};", dbname))
                .unwrap();
            db.query_drop(format!("CREATE DATABASE {};", dbname))
                .unwrap();
            db.query_drop(format!("USE {};", dbname)).unwrap();
            for line in schema.lines() {
                if line.starts_with("--") || line.is_empty() {
                    continue;
                }
                db.query_drop(line).unwrap();
            }
        } else {
            db.query_drop(format!("USE {};", dbname)).unwrap();
        }

        Ok(MySqlBackend {
            handle: db,
            log: log,
            _schema: schema.to_owned(),
            prep_stmts: HashMap::new(),
            db_user: String::from(user),
            db_password: String::from(password),
            db_name: String::from(dbname),
        })
    }

    fn reconnect(&mut self) {
        self.handle = mysql::Conn::new(
            Opts::from_url(&format!(
                "mysql://{}:{}@127.0.0.1/{}",
                self.db_user, self.db_password, self.db_name
            ))
            .unwrap(),
        )
        .unwrap();
    }

    pub fn prep_exec(&mut self, sql: &str, params: Vec<Value>) -> Vec<Vec<Value>> {
        if !self.prep_stmts.contains_key(sql) {
            let stmt = self
                .handle
                .prep(sql)
                .expect(&format!("failed to prepare statement \'{}\'", sql));
            self.prep_stmts.insert(sql.to_owned(), stmt);
        }
        loop {
            match self
                .handle
                .exec_iter(self.prep_stmts[sql].clone(), params.clone())
            {
                Err(e) => {
                    warn!(
                        self.log,
                        "query \'{}\' failed ({}), reconnecting to database", sql, e
                    );
                }
                Ok(res) => {
                    let mut rows = vec![];
                    for row in res {
                        let rowvals = row.unwrap().unwrap();
                        let vals: Vec<Value> = rowvals.iter().map(|v| v.clone().into()).collect();
                        rows.push(vals);
                    }
                    // debug!(self.log, "executed query {}, got {} rows", sql, rows.len());
                    return rows;
                }
            }
            self.reconnect();
        }
    }

    fn do_insert(&mut self, table: &str, vals: Vec<Value>, replace: bool) {
        let op = if replace { "REPLACE" } else { "INSERT" };
        let q = format!(
            "{} INTO {} VALUES ({})",
            op,
            table,
            vals.iter().map(|_| "?").collect::<Vec<&str>>().join(",")
        );
        // debug!(self.log, "executed insert query {} for row {:?}", q, vals);
        while let Err(e) = self.handle.exec_drop(q.clone(), vals.clone()) {
            warn!(
                self.log,
                "failed to insert into {}, query {} ({}), reconnecting to database", table, q, e
            );
            self.reconnect();
        }
    }

    pub fn insert(&mut self, table: &str, vals: Vec<Value>) {
        self.do_insert(table, vals, false);
    }

    pub fn replace(&mut self, table: &str, vals: Vec<Value>) {
        self.do_insert(table, vals, true);
    }
}
