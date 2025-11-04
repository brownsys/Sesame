use crate::backend::MySqlBackend;
use crate::policy::context::*;
use crate::routes::common::{Chat, ChatContext, FromBBoxRow};
use rocket::State;
use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame_mysql::{PConParams, PConRow};
use sesame_rocket::error::SesameRenderResult;
use sesame_rocket::rocket::{get, PConTemplate};
use std::sync::{Arc, Mutex};

#[get("/<name>")]
pub(crate) fn buggy_endpoint(
    name: PCon<String, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: YouChatContext,
) -> SesameRenderResult<PConTemplate> {
    // query all sent chats
    let sent: Vec<Chat> = backend
        .lock()
        .unwrap()
        .handle
        .prep_exec_iter(
            "SELECT * FROM chats WHERE sender = ? ORDER BY time",
            vec![name.clone()],
            context.clone(),
        )
        .unwrap()
        .map(|row_result: mysql::Result<PConRow>| -> Chat {
            let row = row_result.unwrap();
            Chat::from_row(row)
        })
        .collect::<Vec<_>>();

    // query all recieved chats
    let recieved: Vec<Chat> = backend
        .lock()
        .unwrap()
        .handle
        .prep_exec_iter(
            "SELECT * FROM chats ORDER BY time",
            PConParams::Empty,
            context.clone(),
        )
        .unwrap()
        .map(|row_result: mysql::Result<PConRow>| -> Chat {
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

    PConTemplate::render("chat", &ctx, context)
}
