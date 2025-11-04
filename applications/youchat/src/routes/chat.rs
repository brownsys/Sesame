use crate::backend::MySqlBackend;
use crate::policy::context::*;
use crate::routes::common::*;

use rocket::State;
use sesame::error::SesameResult;
use sesame::pcon::PCon;
use sesame::policy::NoPolicy;
use sesame_mysql::{PConQueryResult, PConRow};
use sesame_rocket::rocket::{get, post, PConForm, PConRedirect, PConResponseEnum, PConTemplate};
use std::sync::{Arc, Mutex};

#[get("/<name>")]
pub(crate) fn show_chat(
    name: PCon<String, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: YouChatContext,
) -> PConResponseEnum {
    // check that the user is known
    let mut bg = backend.lock().unwrap();
    let user_res: PConQueryResult<_> = (*bg)
        .handle
        .prep_exec_iter(
            "SELECT * FROM users WHERE user_name = ?",
            vec![name.clone()],
            context.clone(),
        )
        .unwrap();

    if !(user_res.count() > 0) {
        return PConRedirect::to("/login", (), context).into();
    }
    drop(bg);
    // query for all sent chats
    let sent: Vec<Chat> = backend
        .lock()
        .unwrap()
        .handle
        .prep_exec_iter(
            "SELECT * FROM chats WHERE sender = ? AND group_chat is NULL ORDER BY time",
            vec![name.clone()],
            context.clone(),
        )
        .unwrap()
        .map(|row_result: mysql::Result<PConRow>| -> Chat {
            let row = row_result.unwrap();
            Chat::from_row(row)
        })
        .collect::<Vec<_>>();

    // query for all recieved chats
    let recieved: Vec<Chat> = backend
        .lock()
        .unwrap()
        .handle
        .prep_exec_iter(
            "SELECT * FROM chats WHERE recipient = ? AND group_chat is NULL ORDER BY time",
            vec![name.clone()],
            context.clone(),
        )
        .unwrap()
        .map(|row_result: mysql::Result<PConRow>| -> Chat {
            let row = row_result.unwrap();
            Chat::from_row(row)
        })
        .collect::<Vec<_>>();

    // construct the context
    let ctx = ChatContext {
        name: name,
        sent_chats: sent,
        recieved_chats: recieved,
        buggy: false,
    };

    PConTemplate::render("chat", &ctx, context).into()
}

#[post("/<name>/send", data = "<data>")]
pub(crate) fn send(
    name: PCon<String, NoPolicy>,
    data: PConForm<MessageRequest>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: YouChatContext,
) -> SesameResult<PConRedirect> {
    // get timestamp of send
    let time = timestamp();

    // assemble values to insert
    let vals = (
        data.recipient.clone(),
        name.clone(),
        data.content.clone(),
        time.clone(),
    );

    // send insert query to db
    let mut bg = backend.lock().unwrap();
    let _ = (*bg).handle.prep_exec_drop(
        "INSERT INTO chats (recipient, sender, content, time) VALUES (?,?,?,?)",
        vals,
        context.clone(),
    );

    PConRedirect::to("/chat/{}", (&name.clone(),), context)
}
