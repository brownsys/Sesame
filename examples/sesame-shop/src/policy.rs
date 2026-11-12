use rocket::Request;
use rocket::http::Cookie;
use sesame::SesameTypeOut;
use sesame::context::UnprotectedContext;
use sesame::policy::{Reason, SimplePolicy};
use sesame_mysql::{schema_policy, SchemaPolicy};
use sesame_rocket::policy::FrontendPolicy;

use crate::context::ContextData;

// Checks downcast to this; the downcast fails (and the check returns false)
// for Context::empty(), which is what queries issued inside a check use.
type ContextDataOut = <ContextData as SesameTypeOut>::Out;

// DP1: a user's profile can only be viewed by admins or the user themself.
#[schema_policy(table = "user", column = 0)] // id
#[schema_policy(table = "user", column = 1)] // username
#[schema_policy(table = "user", column = 2)] // balance_cents
#[schema_policy(table = "user", column = 3)] // admin
#[derive(Clone)]
pub struct ViewUserPolicy {
    owner: Option<i32>,
}

impl SimplePolicy for ViewUserPolicy {
    fn simple_name(&self) -> String {
        "ViewUserPolicy".to_owned()
    }

    fn simple_check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
        // Logging in stores the user's id as their "user_id" cookie.
        if let Reason::Cookie("user_id") = reason {
            return true;
        }
        let Some(ctx) = context.downcast_ref::<ContextDataOut>() else { return false };
        ctx.admin == Some(true) || (self.owner.is_some() && ctx.user_id == self.owner)
    }

    fn simple_join_direct(&mut self, other: &mut Self) {
        if self.owner != other.owner {
            self.owner = None;
        }
    }
}

impl SchemaPolicy for ViewUserPolicy {
    fn from_row(_table: &str, row: &Vec<mysql::Value>) -> Self {
        // user: (id, username, balance_cents, admin)
        ViewUserPolicy { owner: mysql::from_value(row[0].clone()) }
    }
}

// Required to store a ViewUserPolicy value in a cookie. Data that arrives
// from a request with this policy has no known owner, so only admins may see it.
impl FrontendPolicy for ViewUserPolicy {
    fn from_request<'a, 'r>(_request: &'a Request<'r>) -> Self {
        ViewUserPolicy { owner: None }
    }

    fn from_cookie<'a, 'r>(
        _name: &str,
        _cookie: &'a Cookie<'static>,
        _request: &'a Request<'r>,
    ) -> Self {
        ViewUserPolicy { owner: None }
    }
}

// DP2: a purchase can only be viewed by admins, the buyer, and the seller.
#[schema_policy(table = "purchase", column = 1)] // user_id
#[schema_policy(table = "purchase", column = 2)] // item_id
#[schema_policy(table = "purchase", column = 3)] // quantity
#[schema_policy(table = "purchase", column = 4)] // price_cents
#[derive(Clone)]
pub struct ViewPurchasePolicy {
    buyer: Option<i32>,
    item_id: Option<i32>,
}

impl SimplePolicy for ViewPurchasePolicy {
    fn simple_name(&self) -> String {
        "ViewPurchasePolicy".to_owned()
    }

    fn simple_check(&self, context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
        let Some(ctx) = context.downcast_ref::<ContextDataOut>() else { return false };
        let Some(me) = ctx.user_id else { return false };
        if ctx.admin == Some(true) || self.buyer == Some(me) {
            return true;
        }
        // The seller is not on the purchase row; ask the item.
        match self.item_id {
            Some(item_id) => ctx.db.is_seller(item_id, me),
            None => false,
        }
    }

    fn simple_join_direct(&mut self, other: &mut Self) {
        if self.buyer != other.buyer {
            self.buyer = None;
        }
        if self.item_id != other.item_id {
            self.item_id = None;
        }
    }
}

impl SchemaPolicy for ViewPurchasePolicy {
    fn from_row(_table: &str, row: &Vec<mysql::Value>) -> Self {
        // purchase: (id, user_id, item_id, quantity, price_cents)
        ViewPurchasePolicy {
            buyer: mysql::from_value(row[1].clone()),
            item_id: mysql::from_value(row[2].clone()),
        }
    }
}

// DP3: a purchase can only be added by a user (not an admin) with a
// sufficient balance, and cannot exceed the item's available quantity.
#[derive(Clone)]
pub struct AddPurchasePolicy {}

impl SimplePolicy for AddPurchasePolicy {
    fn simple_name(&self) -> String {
        "AddPurchasePolicy".to_owned()
    }

    fn simple_check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
        let Some(ctx) = context.downcast_ref::<ContextDataOut>() else { return false };
        let (Some(me), Some(balance)) = (ctx.user_id, ctx.balance_cents) else { return false };
        match reason {
            // INSERT INTO purchase (user_id, item_id, quantity, price_cents)
            Reason::DB(query, params) if query.starts_with("INSERT INTO purchase") => {
                let buyer: i32 = mysql::from_value(params[0].clone());
                let item_id: i32 = mysql::from_value(params[1].clone());
                let quantity: i32 = mysql::from_value(params[2].clone());
                let price_cents: i64 = mysql::from_value(params[3].clone());
                let sufficient_balance = price_cents
                    .checked_mul(i64::from(quantity))
                    .is_some_and(|cost| balance >= cost);
                ctx.admin == Some(false)
                    && buyer == me
                    && sufficient_balance
                    && ctx.db.has_stock(item_id, quantity)
            }
            _ => false,
        }
    }

    fn simple_join_direct(&mut self, _other: &mut Self) {}
}

impl FrontendPolicy for AddPurchasePolicy {
    fn from_request<'a, 'r>(_request: &'a Request<'r>) -> Self {
        AddPurchasePolicy {}
    }

    fn from_cookie<'a, 'r>(
        _name: &str,
        _cookie: &'a Cookie<'static>,
        _request: &'a Request<'r>,
    ) -> Self {
        AddPurchasePolicy {}
    }
}

// DP4: an item can be viewed by any logged in admin or user.
#[schema_policy(table = "item", column = 1)] // name
#[schema_policy(table = "item", column = 2)] // price_cents
#[schema_policy(table = "item", column = 3)] // quantity
#[schema_policy(table = "item", column = 4)] // seller_id
#[derive(Clone)]
pub struct ViewItemPolicy {}

impl SimplePolicy for ViewItemPolicy {
    fn simple_name(&self) -> String {
        "ViewItemPolicy".to_owned()
    }

    fn simple_check(&self, context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
        context
            .downcast_ref::<ContextDataOut>()
            .map_or(false, |ctx| ctx.user_id.is_some())
    }

    fn simple_join_direct(&mut self, _other: &mut Self) {}
}

impl SchemaPolicy for ViewItemPolicy {
    fn from_row(_table: &str, _row: &Vec<mysql::Value>) -> Self {
        ViewItemPolicy {}
    }
}

// DP5: an item can only be added by a user, not an admin.
#[derive(Clone)]
pub struct AddItemPolicy {}

impl SimplePolicy for AddItemPolicy {
    fn simple_name(&self) -> String {
        "AddItemPolicy".to_owned()
    }

    fn simple_check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
        let Some(ctx) = context.downcast_ref::<ContextDataOut>() else { return false };
        match reason {
            Reason::DB(query, _) if query.starts_with("INSERT INTO item") => ctx.admin == Some(false),
            _ => false,
        }
    }

    fn simple_join_direct(&mut self, _other: &mut Self) {}
}

impl FrontendPolicy for AddItemPolicy {
    fn from_request<'a, 'r>(_request: &'a Request<'r>) -> Self {
        AddItemPolicy {}
    }

    fn from_cookie<'a, 'r>(
        _name: &str,
        _cookie: &'a Cookie<'static>,
        _request: &'a Request<'r>,
    ) -> Self {
        AddItemPolicy {}
    }
}
