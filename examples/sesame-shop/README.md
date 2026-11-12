# Sesame Tutorial

## Table of Contents

- [Introduction](#introduction)
- [Your Standard Item-Shop](#your-standard-item-shop)
- [Action-Driven Policy vs Sesame's Data-Driven Policy](#action-driven-policy-vs-sesames-data-driven-policy)
- [Integration 1: Defining the Context](#integration-1-defining-the-context)
- [Integration 2: Defining Policies](#integration-2-defining-policies)
- [Integration 3-A: Sesame in the Data Layer](#integration-3-a-sesame-in-the-data-layer)
- [Integration 3-B: Sesame in the API Layer](#integration-3-b-sesame-in-the-api-layer)
- [Integration 3-C: Sesame in the Business Layer: Privacy Regions](#integration-3-c-sesame-in-the-business-layer-privacy-regoins)
  - [Verified Regions](#verified-regions)
  - [Critical Regions](#critical-regions)
  - [Unchecked Critical Regions](#unchecked-critical-regions)
  - [Sandboxes](#sandboxes)

## Introduction

This guide is for developpers who want to protect their application with Sesame. It assumes the developper is using, or is willing to migrate to, frameworks provided by Sesame. Integration of sesame into unsupported frameworks is out of scope for this guide.

Sesame is a security tool that helps developpers avoid accessing objects in unintended manners. Sesame's philosophy is to hide the instantiation of an object of type `T` behind a "private container" `PCon<T, P>`, and attach to it a "policy" `P`. This policy `P` implements a method that, depending on a `Context` and a `Reason` object, signals whether or not the instance of `T` is allowed to be manipulated. Having both a `Context` and a `Reason` object may seem redundant. Their difference, however, highlights how Sesame defines its trust boundary. `Context` is a developper-defined object, usually containing the information one would store in a cookie, whereas `Reason` is a framework-defined object that is not extendable without a custom build of sesame. 

A helpful yet simplified mental model of `PCon` is as follows.
```rs
struct PCon<T, P: Policy> {
    instance: Box<T>,
    policy: P,
}
```

We will use a top-down approach for integrating sesame. That is, starting with the api layer, changing how endpoints are protected, and trickling the changes down to the database connector. Note here that both the endpoint handler framework `rocket` and the SQL database connector `mysql` are both already ported to sesame under the packages `sesame_rocket` and `sesame_mysql` respectively.


## Your Standard Item-Shop

Consider the following simple item shop app we will call Sesame-Shop. 
```rs
// model.rs
use rocket::serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub balance_cents: i64,
    pub admin: bool,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Item {
    pub id: i32,
    pub name: String,
    pub price_cents: i64,
    pub quantity: i32,
    pub seller_id: i32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Purchase {
    pub id: i32,
    pub user_id: i32,
    pub item_id: i32,
    pub quantity: i32,
    pub price_cents: i64,
}

```

```rs
// controller.rs
use rocket::{State, get, post, serde::json::Json};
...

#[derive(Serialize, Deserialize)]
pub struct GetUserPurchasesRequest {
    requestee_id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct GetUserPurchasesResponse {
    purchases: Vec<Purchase>,
}

pub struct LoggedIn(pub User);

#[rocket::async_trait]
impl<'r> FromRequest<'r> for LoggedIn {
    type Error = String;

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let shop = req.guard::<&State<Database>>().await.unwrap();
        match req.cookies().get("user_id").and_then(|c| c.value().parse::<i32>().ok()).and_then(|id| shop.get_user_by_id(id)) {
            Some(user) => Outcome::Success(LoggedIn(user)),
            None => Outcome::Failure((Status::Unauthorized, "not logged in".to_owned())),
        }
    }
}


#[post("/register", format = "json", data = "<request>")]
pub fn register(shop: &State<Database>, request: Json<RegisterRequest>) -> Result<Json<RegisterResponse>, String> { ... }

#[post("/login", format = "json", data = "<request>")]
pub fn login(shop: &State<Database>, me: LoggedIn, request: Json<LoginRequest>) -> Result<Json<LoginResponse>, String> { ... }

#[get("/items")]
pub fn get_items(shop: &State<Database>, me: LoggedIn) -> Result<Json<ItemsResponse>, String> { ... }

#[post("/sell", format = "json", data = "<request>")]
pub fn sell(shop: &State<Database>, me: LoggedIn, request: Json<SellRequest>) -> Result<Json<SellResponse>, String> { ... }

#[post("/purchase", format = "json", data = "<request>")]
pub fn purchase(shop: &State<Database>, me: LoggedIn, request: Json<PurchaseRequest>) -> Result<Json<PurchaseResponse>, String> { ... }

#[post("/get-user-purchases", format = "json", data = "<request>")]
pub fn get_user_purchases(shop: &State<Database>, me: LoggedIn, request: Json<GetUserPurchasesRequest>) -> Result<Json<GetUserPurchasesResponse>, String> { ... }

#[post("/get-item-purchases", format = "json", data = "<request>")]
pub fn get_item_purchases(shop: &State<Database>, me: LoggedIn, request: Json<GetItemPurchasesRequest>) -> Result<Json<GetItemPurchasesResponse>, String> { ... }
```

```rs
// database.rs
use mysql::prelude::*;
use mysql::{Conn, Opts, Value};

...

pub struct Database {
    conn: Mutex<Conn>,
}

impl Database {
    pub fn new(url: &str) -> Self { ... }

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

    pub fn add_user(&self, username: &str, balance_cents: i64, admin: bool) -> i32 { ... }

    pub fn get_user(&self, username: &str) -> User { ... }

    pub fn get_user_by_id(&self, id: i32) -> Option<User> { ... }

    pub fn get_items(&self) -> Vec<Item> { ... }

    pub fn add_item(&self, name: &str, price_cents: i64, quantity: i32, seller_id: i32) -> Item { ... }

    pub fn add_purchase(&self, user_id: i32, item_id: i32, quantity: i32, price_cents: i64) -> Purchase { ... }

    pub fn get_user_purchases(&self, id: i32) -> Vec<Purchase> { ... }

    pub fn get_item_purchases(&self, id: i32) -> Vec<Purchase> { ... }

    pub fn is_seller(&self, item_id: i32, user_id: i32) -> bool { ... }

    pub fn has_stock(&self, item_id: i32, quantity: i32) -> bool { ... }
}
```
Notice that for simplicity's sake, ids in this application double as api keys.

## Action-Driven Policy vs Sesame's Data-Driven Policy

As application developers, we wish to enforce security policies on our shop. Often, these policies dictate the enforcement of the legal privacy policy users agree to upon sign up. For example, a privacy policy might include a clause which claims the application will not share a user's balance with users that are not admins. To implement such policies, most developers opt for a heirarchy of if-else statements at each endpoint. Numerous security breaches have proved that this approach is error-prone and requires high levels of awareness, reasoning, and developper discipline. This problem is amplified in larger applications where access control is more complicated and dependes on complex roles and access tags.

To control access, Sesame adopts a data-driven-policy approach of specifying the policy on the data itself: For every data type instance that enters (through a "source") or exits (through a "sink") a policy, attached to that type, must accompany it. This is the driving principle behind the `PCon<T, P>` object.
For some more intuition, contrast this data-driven-policy approach the traditional action-driven-policy approach. Where the policy, usually defined through a heirarchy of if-else statements, determines whether the user is allowed to access the "view data"/"change data" action.     

For Sesame-Shop, we wish to enforce the following action-driven-policies:

    AP1- A user cannot view any user profile that isn't theirs

    AP2- An admin can view any user profile

    AP3- A user "A" can view purchases they made as a buyer, and purchases other user "B" made on items user "A" is a seller of. 

    AP4- An admin can view any purchase.

    AP5- Users with a sufficient balance can make a purchase. Admins cannot. That user cannot purchase more than the item's available quantity.

    AP6- Any Admin or User can view an item

    AP7- Users can sell an item. Admins cannot.

Thus, we first translate them into data-driven-policies:

    DP1- A user's profile's can only be viewed by admins or the user themself. (AP1, AP2)

    DP2- A purchase can only be viewed by Admins, the "buyer", and the "seller". (AP3, AP4)

    DP3- A purchase can only be added by a User with a sufficient balance but not Admins. The purchase's quantity cannot exceed the item's available quantity. (AP5) 

    DP4- An item can be viewed by any Admin, User. (AP6)

    DP5- An item can only be added by a User and not Admins. (AP7)


Instead of checking if the role provides sufficient permissions to trigger the action, Sesame will trigger the action, and check if the returned data's policy (the policy attached to data exiting the sink) is allowed to exit given the context. That final policy check happens at the end of the execution, reactively, rather than proactively. 

Notice that this fact implies a user can still trigger an illegal endpoint which internally performs ressource-intensive computations but never reads or writes to disk or to network while executing. This is because protecting such an endpoint should not be viewed as a data protection problem. For non-data-access control measures, a developper would still need to rely on other means such as action-driven policy validation.

Lucky for us, all of Sesame-Shop's policies can be formulated from a data-driven-policy perspective. And so we have exactly 5 policies to implement: `ViewUserPolicy`, `ViewPurchasePolicy`, `AddPurchasePolicy`, `ViewItemPolicy`, `AddItemPolicy`. Looking at the schema for each of `User`, `Purchase`, and `Item`, we notice that the context needed to check `ViewUserPolicy`, `ViewItemPolicy`, `AddItemPolicy` is simply the user making the request. However, with `ViewPurchasePolicy`, because determining the "seller" of a `Purchase` the context will also require a database connection in addition to the user making the query. `AddPurchasePolicy` also requires the database connection to inspect the `Item`'s available quantity.

## Integration 1: Defining the Context

With that in mind, let's add our first sesame file `context.rs`.
```rs
// context.rs
use std::sync::Arc;

use rocket::State;
use sesame::SesameType;
use sesame::context::Context;
use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame::verified::VerifiedRegion;
use sesame_rocket::rocket::{FromPConRequest, PConRequest, PConRequestOutcome};

use crate::database::Database;
use crate::policy::ViewUserPolicy;

#[derive(SesameType, Clone)]
#[sesame_out_type(verbatim = [db])]
pub struct ContextData {
    pub user_id: Option<PCon<i32, ViewUserPolicy>>, // None when not logged in
    pub admin: Option<PCon<bool, ViewUserPolicy>>,
    pub balance_cents: Option<PCon<i64, ViewUserPolicy>>,
    pub db: Arc<Database>,
}

#[rocket::async_trait]
impl<'a, 'r> FromPConRequest<'a, 'r> for ContextData {
    type PConError = String;

    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        let db: &State<Arc<Database>> = request.guard().await.unwrap();

        // Ids double as api keys, so the cookie carries no policy. The
        // requester's fields come from their user row instead, and keep
        // that row's ViewUserPolicy.
        let user = request
            .cookies()
            .get::<NoPolicy>("user_id")
            .and_then(|cookie| {
                let id: PCon<String, NoPolicy> = cookie.into();
                id.into_verified(VerifiedRegion::new(|id: String| id.parse::<i32>().ok()))
                    .fold_in()
            })
            .and_then(|id| db.get_user_by_id(id, Context::empty()));

        let (user_id, admin, balance_cents) = match user {
            Some(user) => (Some(user.id), Some(user.admin), Some(user.balance_cents)),
            None => (None, None, None),
        };

        PConRequestOutcome::Success(ContextData {
            user_id,
            admin,
            balance_cents,
            db: db.inner().clone(),
        })
    }
}
```

`context.rs` defines `ContextData`, which is the object that is passed to policy to check if the context allows it to let the data exit. Since we are receiving our requests from rocket, `ContextData` needs to be provided at each request rocket processes. For that, we implement the `FromPConRequest<'a, 'r>` trait for it. In Sesame-Shop, the ContextData need only contain the user id, whether or not they are an administrator, their balance and the database for the reasons mentioned above. 

Notice that the cookie's value is parsed with `into_verified(VerifiedRegion::new(...))` rather than by taking it out of its `PCon`. Sesame only lets `discard_box()` unwrap a `PCon<T, NoPolicy>`; for every other policy, code has to run on the data through a region. A verified region runs a pure closure on the inner value and returns a `PCon` with the same policy, so the result is still protected. `fold_in()` then turns the resulting `PCon<Option<i32>, _>` into an `Option<PCon<i32, _>>`. A critical region, in contrast, checks the policy against the context and hands back the raw value, and is reserved for side effects such as sending an email.

With context defined, we are ready to implement our 5 policies in `policy.rs`, let's start with DP2 `ViewPurchasePolicy` by adding some imports.

## Integration 2: Defining Policies

```rs
// policy.rs
use rocket::Request;
use rocket::http::Cookie;
use sesame::SesameTypeOut;
use sesame::context::UnprotectedContext;
use sesame::policy::{Reason, SimplePolicy};
use sesame_mysql::{schema_policy, SchemaPolicy};
use sesame_rocket::policy::FrontendPolicy;

use crate::context::ContextData;
```

To be validated against a context (requester's id and whether or not they're an admin), `ViewPurchasePolicy` needs to know the buyer of said purchase, and the item being purchased. 
```rs
// policy.rs
{ ... }

pub struct ViewPurchasePolicy {
    buyer: Option<i32>,
    item_id: Option<i32>,
}
```

Next, we need it to implement the trait `SimplePolicy`, which entails implementing 3 methods. The first, `fn simple_name(&self) -> String` is used for log auditing purposes. The second, `fn simple_check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool` performs the policy check against our `context` (and a framework-provided `reason` describing the sink, which `ViewPurchasePolicy` does not need). The third, `fn simple_join_direct(&mut self, other: &mut Self)` informs Sesame how to merge two `ViewPurchasePolicy`. This can be triggered, for example, when developpers want to turn a `Vec<PCon<T,_>>` into a `PCon<Vec<T>, _>`.

In Sesame-Shop, `simple_check` checks that the user is either an admin or is the buyer or is the seller. 
```rs
// Checks downcast to this; the downcast fails (and the check returns false)
// for Context::empty(), which is what queries issued inside a check use.
type ContextDataOut = <ContextData as SesameTypeOut>::Out;

// DP2: a purchase can only be viewed by admins, the buyer, and the seller.
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
```

This policy will now trigger on a data sink exit, but so far, contructing the PCon object that uses this policy is still a developper chore at every database call. Fortunetly, Sesame provides a `schema_policy` macro and a `SchemaPolicy` trait that allows sesame to automatically wrap objects from a `sesame_mysql` database into `PCon`s rather than rely on the developper. Intuitively, each `schema_policy` macro is protecting one column of the database when it is fetched.

```rs
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
    ...
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
```

## Integration 3-A: Sesame in the Data Layer 
This entails changing `model.rs` for consistency with the above. There we need to mirror the policy on each column and make `Purchase` implement the `From<PConRow>` trait.
```rs
use sesame::SesameType;
use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame_mysql::{from_value, PConRow};
use sesame_rocket::rocket::ResponsePConJson;

use crate::policy::ViewPurchasePolicy;


#[derive(SesameType, ResponsePConJson, Clone)]
pub struct Purchase {
    pub id: PCon<i32, NoPolicy>,
    pub user_id: PCon<i32, ViewPurchasePolicy>,
    pub item_id: PCon<i32, ViewPurchasePolicy>,
    pub quantity: PCon<i32, ViewPurchasePolicy>,
    pub price_cents: PCon<i64, ViewPurchasePolicy>,
}

// Each column comes back boxed with the policy its #[schema_policy] registers.
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
```

We have thus implement our first policy: `ViewPurchasePolicy`. `ViewPurchasePolicy` is a read policy. DP3's `AddPurchasePolicy`, in contrast, is a write policy. `ViewPurchasePolicy` needs to maintain a state to declare what it is policing. Whereas `AddPurchasePolicy` does not have a meaningful state because it is created following user-generated data rather than database-generated data. Rather than `SchemaPolicy`, we will implement `FrontendPolicy` for `AddPurchasePolicy`. Its `simple_check` is triggered at a sink, which in this case is a database insert call. We intercept that call to check if its logic is compliant. At a database sink, the `reason` is `Reason::DB(query, params)`: the SQL statement and the values bound to it. `AddPurchasePolicy` uses the statement to recognize a purchase insert and reads the purchase from the parameters, so `database.rs` must issue exactly `INSERT INTO purchase (user_id, item_id, quantity, price_cents) VALUES (?, ?, ?, ?)`. Any other spelling or column order makes the check reject the write.

```rs
// policy.rs
{ ... }

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
```

The remaining policies DP1 `ViewUserPolicy`, DP4 `ViewItemPolicy`, and DP5 `AddItemPolicy` are constructed in a similar fashion. 
Implementing these, and putting everything together we get [model.rs](./src/model.rs) and [policy.rs](./src/policy.rs).

We now reflect the necessary changes to `database.rs`. Essentially, this entails drilling a `Context<ContextData>` object so that `SesameConn` is able to use it when validating policies.
```rs
// database.rs
use std::sync::Mutex;

use mysql::prelude::*;
use mysql::{Conn, Opts};
use sesame::context::Context;
use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame_mysql::{PConOpts, SesameConn};

use crate::context::ContextData;
use crate::model::{Item, Purchase, User};
use crate::policy::{AddItemPolicy, AddPurchasePolicy, ViewItemPolicy, ViewUserPolicy};

pub struct Database {
    conn: Mutex<SesameConn>,
    policy_conn: Mutex<Conn>,
}

impl Database {
    pub fn new(url: &str) -> Self { ... }

    pub fn create_tables_if_not_exists(&self) { ... }

    pub fn add_user(
        &self,
        username: PCon<String, NoPolicy>,
        balance_cents: PCon<i64, NoPolicy>,
        admin: PCon<bool, NoPolicy>,
        context: Context<ContextData>,
    ) -> i32 { ... }

    pub fn get_user(&self, username: PCon<String, NoPolicy>, context: Context<ContextData>) -> Option<User> { ... }

    pub fn get_user_by_id(&self, id: PCon<i32, NoPolicy>, context: Context<ContextData>) -> Option<User> { ... }

    pub fn get_items(&self, context: Context<ContextData>) -> Vec<Item> { ... }

    pub fn get_item(&self, id: PCon<i32, NoPolicy>, context: Context<ContextData>) -> Option<Item> { ... }

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
            context,
        )
        .map_err(|e| e.to_string())?;
        ...
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
            context,
        )
        .map_err(|e| e.to_string())?;
        ...
    }

    pub fn get_user_purchases(&self, id: PCon<i32, NoPolicy>, context: Context<ContextData>) -> Vec<Purchase> { ... }

    pub fn get_item_purchases(&self, id: PCon<i32, NoPolicy>, context: Context<ContextData>) -> Vec<Purchase> { ... }

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
```

Notice that if we were to use the `conn` connection for `is_seller` and `has_stock`, `database.rs` would deadlock when checking `AddPurchasePolicy`. This is because the database call invokes a policy check which, itself, needs to make another database call. For that reason, we modified our `database.rs` to contain two database connections and split the policy methods from the application connection. More idiomatic methods to mitigate this problem are unknown to the author at the at the time of writing. 


## Integration 3-B: Sesame in the API Layer 

Jumping up a layer, we need to reflect the changes in `controller.rs`. Which boils down to defining a cookie-handler object we'll call `LoggedIn`, and implementing a `FromPConRequest` trait for it. You can find an implemented example endpoint below for [controller.rs](./src/controller.rs) 

```rs
// controller.rs
[...]

pub struct LoggedIn {
    pub user_id: PCon<i32, ViewUserPolicy>,
}

#[rocket::async_trait]
impl<'a, 'r> FromPConRequest<'a, 'r> for LoggedIn {
    type PConError = String;

    async fn from_pcon_request(
        request: PConRequest<'a, 'r>,
    ) -> PConRequestOutcome<Self, Self::PConError> {
        let data: ContextData = request.guard().await.unwrap();
        match data.user_id {
            Some(user_id) => PConRequestOutcome::Success(LoggedIn { user_id }),
            None => PConRequestOutcome::Failure((Status::Unauthorized, "not logged in".to_owned())),
        }
    }
}

[...]

#[derive(RequestPConJson)]
pub struct SellRequest {
    name: PCon<String, AddItemPolicy>,
    price_cents: PCon<i64, AddItemPolicy>,
    quantity: PCon<i32, AddItemPolicy>,
}

#[derive(ResponsePConJson)]
pub struct SellResponse {
    item: Item,
}

#[post("/sell", data = "<request>")]
pub fn sell(
    shop: &State<Arc<Database>>,
    me: LoggedIn,
    request: PConJson<SellRequest>,
    context: Context<ContextData>,
) -> Result<JsonResponse<SellResponse, ContextData>, String> {
    let request = request.into_inner();
    let item = shop.add_item(
        request.name,
        request.price_cents,
        request.quantity,
        me.user_id,
        context.clone(),
    )?;
    Ok(JsonResponse::from((SellResponse { item }, context)))
}
```

Finally, `main.rs` creates the database, hands it to rocket as managed state, and mounts the endpoints.
```rs
// main.rs
mod context;
mod controller;
mod database;
mod model;
mod policy;

use std::sync::Arc;

use sesame_rocket::rocket::{routes, SesameRocket};

use crate::database::Database;

#[rocket::main]
async fn main() {
    let db = Database::new("mysql://root:password@127.0.0.1:3306/shop");
    db.create_tables_if_not_exists();

    let result = SesameRocket::build()
        .manage(Arc::new(db))
        .mount(
            "/",
            routes![
                controller::register,
                controller::login,
                controller::get_items,
                controller::sell,
                controller::purchase,
                controller::get_user_purchases,
                controller::get_item_purchases,
            ],
        )
        .launch()
        .await;

    if let Err(e) = result {
        eprintln!("rocket failed to launch: {e}");
    }
}
```

## Integration 3-C: Sesame in the Business Layer: Privacy Regoins

Often times, while processing a request, one might wish to manipulate data beyond simply sending it to the database. This includes performing business logic on said data that may run in the same process (e.g. multiplication by 2), or might run on a different machine (e.g. sending the data to an external exchange-rate conversion service). In each of these cases, Sesame's `PCon<_,_>` prevents the developer from accessing or releasing the underlying data. Fortunetly, Sesame provides tools that re-enable this flexibility: Regions.

Sesame defines four types of regions: Verified Regions, Critical Regions, Unchecked Critical Regions, and Sandboxes. 

### Verified Regions

```rs
use sesame::verified::VerifiedRegion;

// Format a purchase's price for display; the result is still protected by ViewPurchasePolicy.
let price: PCon<String, ViewPurchasePolicy> = purchase.price_cents
    .into_verified(VerifiedRegion::new(|cents: i64| format!("${}.{:02}", cents / 100, cents % 100)));
```

Verified regions are used for pure computation over PCon-protected data. The closure's result is re-boxed under the same policy, so no policy check is needed. They are only "verified" when Scrutinizer is run. Scrutinizer statically verifies the closure is pure (no I/O, no globals, and no `println!`). See [SCRUTINIZER.md](../../SCRUTINIZER.md).

### Critical Regions

```rs
use sesame::critical::{CriticalRegion, Signature};

// Email the buyer a receipt. Runs only if ViewPurchasePolicy allows it in this context.
let sent = purchase.price_cents.into_critical(
    context.clone(),
    CriticalRegion::new(
        |cents: i64, email: String| send_receipt(&email, cents),
        Signature { username: "reviewer", signature: "..." },
    ),
    buyer_email,
);
```

(Checked) Critical Regions are used when data has to leave Sesame through means that are not a Sesame-provided library (e.i. Sesame has no sink for). The policy is checked before the closure is run. Unlike Verified Regions, the closure's return is not re-boxed into a PCon. This fact, and the fact that one could arbitrarly share data make (Checked) Critical Regions require the signature of a reviewer. 

### Unchecked Critical Regions

```rs
use sesame::critical::{Signature, UncheckedCriticalRegion};

// Migration script: copy every purchase into the archive, regardless of policy.
purchase.price_cents.into_critical_unchecked(
    UncheckedCriticalRegion::new(
        |cents: i64, policy: ViewPurchasePolicy, archive: &Archive| archive.write(cents, policy),
        Signature { username: "reviewer", signature: "..." },
    ),
    &archive,
);
```

Same as a critical region but with no policy check. The closure gets the raw value *and* the policy object, so it can re-box, inspect, or store the policy.

### Sandboxes

```rs
// In a separate sandbox crate, compiled to WASM.
use sesame::sandbox::SesameSandbox;

#[SesameSandbox()]
pub fn receipt_hash(inputs: (String, i64)) -> String {
    // Arbitrary, unverified code (e.g. a third-party crate) runs here.
    sha256(format!("{}:{}", inputs.0, inputs.1))
}
```
```rs
// In the application.
use sesame::policy::{AnyPolicy, AnyPolicyDyn};
use sesame::sandbox::execute_sandbox;

let hash: PCon<String, AnyPolicy> =
    execute_sandbox::<receipt_hash, _, _, dyn AnyPolicyDyn>((buyer_email, purchase.price_cents));
```

Sandboxes are verified region that are pure but couldn't be proven pure by Scrutinizer. Sandboxed regions instead compile the code to an isolated environment (WASM) where the function runs without access to IO, network, memory, etc...  

See the [sandbox build instructions](../../sesame/build/README.md) and the [sandbox examples](../sandbox/).

