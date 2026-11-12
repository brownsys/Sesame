use std::sync::Mutex;

use mysql::prelude::*;
use mysql::{Conn, Opts};
use sesame::context::Context;
use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame_mysql::{from_value, PConOpts, PConParams, PConRow, SesameConn};

use crate::context::ContextData;
use crate::model::{Item, Purchase, User};
use crate::policy::{AddItemPolicy, AddPurchasePolicy, ViewItemPolicy, ViewUserPolicy};

pub struct Database {
    conn: Mutex<SesameConn>,
    policy_conn: Mutex<Conn>,
}

impl Database {
    pub fn new(url: &str) -> Self {
        let conn = SesameConn::new(PConOpts::from_url(url).expect("invalid database URL"))
            .expect("cannot connect to the database");
        let policy_conn = Conn::new(Opts::from_url(url).expect("invalid database URL"))
            .expect("cannot connect to the database");
        Self {
            conn: Mutex::new(conn),
            policy_conn: Mutex::new(policy_conn),
        }
    }

    pub fn create_tables_if_not_exists(&self) {
        let mut conn = self.conn.lock().unwrap();
        conn.query_drop(
            "CREATE TABLE IF NOT EXISTS user (
                id INT PRIMARY KEY AUTO_INCREMENT,
                username VARCHAR(50) NOT NULL UNIQUE,
                balance_cents BIGINT NOT NULL,
                admin BOOLEAN NOT NULL
            )",
        )
        .unwrap();
        conn.query_drop(
            "CREATE TABLE IF NOT EXISTS item (
                id INT PRIMARY KEY AUTO_INCREMENT,
                name VARCHAR(50) NOT NULL,
                price_cents BIGINT NOT NULL,
                quantity INT NOT NULL,
                seller_id INT NOT NULL,
                FOREIGN KEY (seller_id) REFERENCES user(id)
            )",
        )
        .unwrap();
        conn.query_drop(
            "CREATE TABLE IF NOT EXISTS purchase (
                id INT PRIMARY KEY AUTO_INCREMENT,
                user_id INT NOT NULL,
                item_id INT NOT NULL,
                quantity INT NOT NULL,
                price_cents BIGINT NOT NULL,
                FOREIGN KEY (user_id) REFERENCES user(id),
                FOREIGN KEY (item_id) REFERENCES item(id)
            )",
        )
        .unwrap();
    }

    // Runs a query on the application connection and converts every row.
    fn query<T: From<PConRow>, P: Into<PConParams>>(
        conn: &mut SesameConn,
        sql: &str,
        params: P,
        context: Context<ContextData>,
    ) -> Vec<T> {
        conn.exec_iter(sql, params, context)
            .unwrap_or_else(|e| panic!("query '{sql}' failed: {e}"))
            .map(|row| T::from(row.unwrap()))
            .collect()
    }

    pub fn add_user(
        &self,
        username: PCon<String, NoPolicy>,
        balance_cents: PCon<i64, NoPolicy>,
        admin: PCon<bool, NoPolicy>,
        context: Context<ContextData>,
    ) -> i32 {
        let mut conn = self.conn.lock().unwrap();
        conn.exec_drop(
            "INSERT INTO user (username, balance_cents, admin) VALUES (?, ?, ?)",
            (username, balance_cents, admin),
            context,
        )
        .expect("cannot add user");
        // LAST_INSERT_ID() belongs to no table, so it carries no policy.
        let row = conn
            .query_iter("SELECT LAST_INSERT_ID()")
            .expect("cannot read user id")
            .next()
            .unwrap()
            .unwrap();
        let id: PCon<i32, NoPolicy> = from_value(row.get(0).unwrap()).unwrap();
        id.discard_box()
    }

    pub fn get_user(&self, username: PCon<String, NoPolicy>, context: Context<ContextData>) -> Option<User> {
        let mut conn = self.conn.lock().unwrap();
        Self::query(&mut conn, "SELECT * FROM user WHERE username = ?", (username,), context)
            .into_iter()
            .next()
    }

    pub fn get_user_by_id(&self, id: PCon<i32, NoPolicy>, context: Context<ContextData>) -> Option<User> {
        let mut conn = self.conn.lock().unwrap();
        Self::query(&mut conn, "SELECT * FROM user WHERE id = ?", (id,), context)
            .into_iter()
            .next()
    }

    pub fn get_items(&self, context: Context<ContextData>) -> Vec<Item> {
        let mut conn = self.conn.lock().unwrap();
        Self::query(&mut conn, "SELECT * FROM item", (), context)
    }

    pub fn get_item(&self, id: PCon<i32, NoPolicy>, context: Context<ContextData>) -> Option<Item> {
        let mut conn = self.conn.lock().unwrap();
        Self::query(&mut conn, "SELECT * FROM item WHERE id = ?", (id,), context)
            .into_iter()
            .next()
    }

    pub fn add_item(
        &self,
        name: PCon<String, AddItemPolicy>,
        price_cents: PCon<i64, AddItemPolicy>,
        quantity: PCon<i32, AddItemPolicy>,
        seller_id: PCon<i32, ViewUserPolicy>,
        context: Context<ContextData>,
    ) -> Result<Item, String> {
        let mut conn = self.conn.lock().unwrap();
        // AddItemPolicy matches on this exact statement.
        conn.exec_drop(
            "INSERT INTO item (name, price_cents, quantity, seller_id) VALUES (?, ?, ?, ?)",
            (name, price_cents, quantity, seller_id),
            context.clone(),
        )
        .map_err(|e| e.to_string())?;
        Self::query(&mut conn, "SELECT * FROM item WHERE id = LAST_INSERT_ID()", (), context)
            .into_iter()
            .next()
            .ok_or_else(|| "Item was not inserted".to_owned())
    }

    pub fn add_purchase(
        &self,
        user_id: PCon<i32, ViewUserPolicy>,
        item_id: PCon<i32, NoPolicy>,
        quantity: PCon<i32, AddPurchasePolicy>,
        price_cents: PCon<i64, ViewItemPolicy>,
        context: Context<ContextData>,
    ) -> Result<Purchase, String> {
        let mut conn = self.conn.lock().unwrap();
        // AddPurchasePolicy matches on this exact statement and parameter order.
        conn.exec_drop(
            "INSERT INTO purchase (user_id, item_id, quantity, price_cents) VALUES (?, ?, ?, ?)",
            (user_id, item_id, quantity, price_cents),
            context.clone(),
        )
        .map_err(|e| e.to_string())?;
        Self::query(&mut conn, "SELECT * FROM purchase WHERE id = LAST_INSERT_ID()", (), context)
            .into_iter()
            .next()
            .ok_or_else(|| "Purchase was not inserted".to_owned())
    }

    pub fn get_user_purchases(&self, id: PCon<i32, NoPolicy>, context: Context<ContextData>) -> Vec<Purchase> {
        let mut conn = self.conn.lock().unwrap();
        Self::query(&mut conn, "SELECT * FROM purchase WHERE user_id = ?", (id,), context)
    }

    pub fn get_item_purchases(&self, id: PCon<i32, NoPolicy>, context: Context<ContextData>) -> Vec<Purchase> {
        let mut conn = self.conn.lock().unwrap();
        Self::query(&mut conn, "SELECT * FROM purchase WHERE item_id = ?", (id,), context)
    }

    // Policy lookups: called from inside policy checks, so they use plain
    // values and the separate policy connection, and only ever return a bool.
    pub fn is_seller(&self, item_id: i32, user_id: i32) -> bool {
        let mut conn = self.policy_conn.lock().unwrap();
        conn.exec_first::<i32, _, _>("SELECT id FROM item WHERE id = ? AND seller_id = ?", (item_id, user_id))
            .unwrap()
            .is_some()
    }

    pub fn has_stock(&self, item_id: i32, quantity: i32) -> bool {
        let mut conn = self.policy_conn.lock().unwrap();
        conn.exec_first::<i32, _, _>("SELECT id FROM item WHERE id = ? AND quantity >= ?", (item_id, quantity))
            .unwrap()
            .is_some()
    }
}
