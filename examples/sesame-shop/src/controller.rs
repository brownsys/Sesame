use std::collections::HashMap; // used by the ResponsePConJson derive
use std::sync::Arc;

use rocket::State;
use rocket::http::Status;
use sesame::context::Context;
use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame::verified::VerifiedRegion;
use sesame_rocket::rocket::{
    get, post, FromPConRequest, JsonResponse, PConCookie, PConCookieJar, PConJson, PConRequest,
    PConRequestOutcome, RequestPConJson, ResponsePConJson,
};

use crate::context::ContextData;
use crate::database::Database;
use crate::model::{Item, Purchase};
use crate::policy::{AddItemPolicy, AddPurchasePolicy, ViewUserPolicy};

// The requester's id, for endpoints that write it into a new row.
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

#[derive(RequestPConJson)]
pub struct RegisterRequest {
    username: PCon<String, NoPolicy>,
    admin: PCon<bool, NoPolicy>,
}

#[derive(ResponsePConJson)]
pub struct RegisterResponse {
    user_id: i32,
}

#[post("/register", data = "<request>")]
pub fn register(
    shop: &State<Arc<Database>>,
    request: PConJson<RegisterRequest>,
    context: Context<ContextData>,
) -> JsonResponse<RegisterResponse, ContextData> {
    let request = request.into_inner();
    let balance_cents = PCon::new(0, NoPolicy {});
    let user_id = shop.add_user(request.username, balance_cents, request.admin, context.clone());
    JsonResponse::from((RegisterResponse { user_id }, context))
}

#[derive(RequestPConJson)]
pub struct LoginRequest {
    username: PCon<String, NoPolicy>,
}

#[post("/login", data = "<request>")]
pub fn login(
    shop: &State<Arc<Database>>,
    cookies: PConCookieJar<'_, '_>,
    request: PConJson<LoginRequest>,
    context: Context<ContextData>,
) -> Result<(), String> {
    let user = shop
        .get_user(request.into_inner().username, context.clone())
        .ok_or_else(|| "User not found".to_owned())?;
    // The id stays under ViewUserPolicy, which allows it to become the
    // requester's "user_id" cookie.
    let value = user.id.into_verified(VerifiedRegion::new(|id: i32| id.to_string()));
    cookies
        .add(PConCookie::build("user_id", value).path("/").finish(), context)
        .map_err(|_| "Cannot set cookie".to_owned())
}

#[derive(ResponsePConJson)]
pub struct ItemsResponse {
    items: Vec<Item>,
}

#[get("/items")]
pub fn get_items(
    shop: &State<Arc<Database>>,
    context: Context<ContextData>,
) -> JsonResponse<ItemsResponse, ContextData> {
    let items = shop.get_items(context.clone());
    JsonResponse::from((ItemsResponse { items }, context))
}

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

#[derive(RequestPConJson)]
pub struct PurchaseRequest {
    item_id: PCon<i32, NoPolicy>,
    quantity: PCon<i32, AddPurchasePolicy>,
}

#[derive(ResponsePConJson)]
pub struct PurchaseResponse {
    purchase: Purchase,
}

#[post("/purchase", data = "<request>")]
pub fn purchase(
    shop: &State<Arc<Database>>,
    me: LoggedIn,
    request: PConJson<PurchaseRequest>,
    context: Context<ContextData>,
) -> Result<JsonResponse<PurchaseResponse, ContextData>, String> {
    let request = request.into_inner();
    let item = shop
        .get_item(request.item_id.clone(), context.clone())
        .ok_or_else(|| "Item not found".to_owned())?;
    let purchase = shop.add_purchase(
        me.user_id,
        request.item_id,
        request.quantity,
        item.price_cents,
        context.clone(),
    )?;
    Ok(JsonResponse::from((PurchaseResponse { purchase }, context)))
}

#[derive(RequestPConJson)]
pub struct GetUserPurchasesRequest {
    requestee_id: PCon<i32, NoPolicy>,
}

#[derive(ResponsePConJson)]
pub struct GetUserPurchasesResponse {
    purchases: Vec<Purchase>,
}

#[post("/get-user-purchases", data = "<request>")]
pub fn get_user_purchases(
    shop: &State<Arc<Database>>,
    request: PConJson<GetUserPurchasesRequest>,
    context: Context<ContextData>,
) -> JsonResponse<GetUserPurchasesResponse, ContextData> {
    let purchases = shop.get_user_purchases(request.into_inner().requestee_id, context.clone());
    JsonResponse::from((GetUserPurchasesResponse { purchases }, context))
}

#[derive(RequestPConJson)]
pub struct GetItemPurchasesRequest {
    item_id: PCon<i32, NoPolicy>,
}

#[derive(ResponsePConJson)]
pub struct GetItemPurchasesResponse {
    purchases: Vec<Purchase>,
}

#[post("/get-item-purchases", data = "<request>")]
pub fn get_item_purchases(
    shop: &State<Arc<Database>>,
    request: PConJson<GetItemPurchasesRequest>,
    context: Context<ContextData>,
) -> JsonResponse<GetItemPurchasesResponse, ContextData> {
    let purchases = shop.get_item_purchases(request.into_inner().item_id, context.clone());
    JsonResponse::from((GetItemPurchasesResponse { purchases }, context))
}
