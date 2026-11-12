use std::collections::HashMap; // used by the ResponsePConJson derive

use sesame::SesameType;
use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame_mysql::{from_value, PConRow};
use sesame_rocket::rocket::ResponsePConJson;

use crate::policy::{ViewItemPolicy, ViewPurchasePolicy, ViewUserPolicy};

#[derive(SesameType, ResponsePConJson, Clone)]
pub struct User {
    pub id: PCon<i32, ViewUserPolicy>,
    pub username: PCon<String, ViewUserPolicy>,
    pub balance_cents: PCon<i64, ViewUserPolicy>,
    pub admin: PCon<bool, ViewUserPolicy>,
}

#[derive(SesameType, ResponsePConJson, Clone)]
pub struct Item {
    pub id: PCon<i32, NoPolicy>,
    pub name: PCon<String, ViewItemPolicy>,
    pub price_cents: PCon<i64, ViewItemPolicy>,
    pub quantity: PCon<i32, ViewItemPolicy>,
    pub seller_id: PCon<i32, ViewItemPolicy>,
}

#[derive(SesameType, ResponsePConJson, Clone)]
pub struct Purchase {
    pub id: PCon<i32, NoPolicy>,
    pub user_id: PCon<i32, ViewPurchasePolicy>,
    pub item_id: PCon<i32, ViewPurchasePolicy>,
    pub quantity: PCon<i32, ViewPurchasePolicy>,
    pub price_cents: PCon<i64, ViewPurchasePolicy>,
}

// Each column comes back boxed with the policy its #[schema_policy] registers.
impl From<PConRow> for User {
    fn from(row: PConRow) -> Self {
        User {
            id: from_value(row.get(0).unwrap()).unwrap(),
            username: from_value(row.get(1).unwrap()).unwrap(),
            balance_cents: from_value(row.get(2).unwrap()).unwrap(),
            admin: from_value(row.get(3).unwrap()).unwrap(),
        }
    }
}

impl From<PConRow> for Item {
    fn from(row: PConRow) -> Self {
        Item {
            id: from_value(row.get(0).unwrap()).unwrap(),
            name: from_value(row.get(1).unwrap()).unwrap(),
            price_cents: from_value(row.get(2).unwrap()).unwrap(),
            quantity: from_value(row.get(3).unwrap()).unwrap(),
            seller_id: from_value(row.get(4).unwrap()).unwrap(),
        }
    }
}

impl From<PConRow> for Purchase {
    fn from(row: PConRow) -> Self {
        Purchase {
            id: from_value(row.get(0).unwrap()).unwrap(),
            user_id: from_value(row.get(1).unwrap()).unwrap(),
            item_id: from_value(row.get(2).unwrap()).unwrap(),
            quantity: from_value(row.get(3).unwrap()).unwrap(),
            price_cents: from_value(row.get(4).unwrap()).unwrap(),
        }
    }
}
