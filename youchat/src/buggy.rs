use rocket::State;
use std::sync::{Mutex, Arc};
use alohomora::{bbox::BBox, policy::NoPolicy, db::BBoxParams, rocket::BBoxTemplate};
use alohomora::db::BBoxRow; 
use alohomora_derive::get;
use crate::backend::MySqlBackend;
use crate::common::{ChatContext, Chat, FromBBoxRow};
use crate::context::*;

#[get("/<name>")]
pub(crate) fn buggy_endpoint(
    name: BBox<String, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: YouChatContext,
) -> BBoxTemplate {
    // query all sent chats
    let sent: Vec<Chat> = backend.lock().unwrap().handle
        .prep_exec_iter(
            "SELECT * FROM chats WHERE sender = ? ORDER BY time", 
            vec![name.clone()], 
            context.clone())
        .unwrap()
        .map(|row_result: mysql::Result<BBoxRow> | -> Chat {
            let row = row_result.unwrap(); 
            Chat::from_row(row)
        })
        .collect::<Vec<_>>(); 

    // query all recieved chats
    let recieved: Vec<Chat> = backend.lock().unwrap().handle.prep_exec_iter(
        "SELECT * FROM chats ORDER BY time", 
        BBoxParams::Empty, 
        context.clone())
        .unwrap()
        .map(|row_result: mysql::Result<BBoxRow> | -> Chat {
            let row = row_result.unwrap(); 
            Chat::from_row(row)
        })
        .collect::<Vec<_>>(); 

    
    // make context
    let ctx = ChatContext {
        name: name,
        sent_chats: sent,
        recieved_chats: recieved,
        buggy: true,
    };

    BBoxTemplate::render("chat", &ctx, context)
}
