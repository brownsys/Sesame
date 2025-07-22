use rocket::State;
use std::sync::{Mutex, Arc};
use alohomora::{bbox::BBox, policy::NoPolicy};
use alohomora::rocket::{get, post, BBoxRedirect, BBoxTemplate, BBoxForm, BBoxResponseEnum};
use alohomora::db::{BBoxRow, BBoxQueryResult}; 
use crate::backend::MySqlBackend;
use crate::common::*;
use crate::context::*;


use std::any::Any;
use core::fmt::Debug;
use alohomora::dyns::{SesameDynType, SesameTypeDynTypes};
// Ideally, this becomes a macro.
pub trait AnyDebug: Debug + Any {
    fn upcast_any(&self) -> &dyn Any;
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any>;
    fn upcast_debug(&self) -> &dyn Debug;
    fn upcast_debug_box(self: Box<Self>) -> Box<dyn Debug>;

}
impl<T: Debug + Any> AnyDebug for T {
    fn upcast_any(&self) -> &dyn Any { self }
    fn upcast_any_box(self: Box<Self>) -> Box<dyn Any> { Box::new(*self) }
    fn upcast_debug(&self) -> &dyn Debug { self }
    fn upcast_debug_box(self: Box<Self>) -> Box<dyn Debug> { Box::new(*self) }
}
// End of Macro.

// Ideally, this is also a derive macro (or maybe directly part of previous macro).
impl SesameDynType for dyn AnyDebug {
    fn upcast_ref(&self) -> &dyn Any {
        self.upcast_any()
    }
    fn upcast_box(self: Box<Self>) -> Box<dyn Any> {
        self.upcast_any_box()
    }
}

impl<T: AnyDebug> SesameTypeDynTypes<T> for dyn AnyDebug {
    fn box_me(self) -> Box<dyn AnyDebug> {
        Box::new(self)
    }
}
// End of Macro.


#[get("/<name>")]
pub(crate) fn show_chat(name: BBox<String, NoPolicy>, 
                        backend: &State<Arc<Mutex<MySqlBackend>>>,
                        context: YouChatContext) -> BBoxResponseEnum {
    // check that the user is known
    let mut bg = backend.lock().unwrap();
    let user_res: BBoxQueryResult<_> = (*bg).handle
        .prep_exec_iter(
            "SELECT * FROM users WHERE user_name = ?", 
            vec![name.clone()], 
            context.clone())
        .unwrap(); 

    if !(user_res.count() > 0) { 
        return BBoxResponseEnum::Redirect(BBoxRedirect::to("/login", (), context));
     }
    drop(bg);
    // query for all sent chats
    let sent: Vec<Chat> = backend.lock().unwrap().handle
        .prep_exec_iter(
            "SELECT * FROM chats WHERE sender = ? AND group_chat is NULL ORDER BY time", 
            vec![name.clone()], 
            context.clone())
        .unwrap()
        .map(|row_result: mysql::Result<BBoxRow> | -> Chat {
            let row = row_result.unwrap(); 
            Chat::from_row(row)
        })
        .collect::<Vec<_>>(); 

    // query for all recieved chats
    let recieved: Vec<Chat> = backend.lock().unwrap().handle
        .prep_exec_iter(
            "SELECT * FROM chats WHERE recipient = ? AND group_chat is NULL ORDER BY time", 
            vec![name.clone()], 
            context.clone())
        .unwrap()
        .map(|row_result: mysql::Result<BBoxRow> | -> Chat {
            let row = row_result.unwrap(); 
            Chat::from_row(row)
        })
        .collect::<Vec<_>>(); 
    
    // construct the context
    let ctx = ChatContext {
        name: name,
        sent_chats: sent,
        recieved_chats: recieved,
        buggy: false
    };

    BBoxResponseEnum::Template(BBoxTemplate::render("chat", &ctx, context))
}

#[post("/<name>/send", data = "<data>")]
pub(crate) fn send(name: BBox<String, NoPolicy>, 
                   data: BBoxForm<MessageRequest>,
                   backend: &State<Arc<Mutex<MySqlBackend>>>,
                   context: YouChatContext) -> BBoxRedirect {
    // get timestamp of send
    let time = timestamp();
    
    // assemble values to insert
    let vals = (
        data.recipient.clone(), 
        name.clone(),
        data.content.clone(),
        time.clone()
    );

    // send insert query to db
    let mut bg = backend.lock().unwrap();
    let _ = (*bg).handle
        .prep_exec_drop(
            "INSERT INTO chats (recipient, sender, content, time) VALUES (?,?,?,?)", 
            vals, 
            context.clone());

    BBoxRedirect::to("/chat/{}", (&name.clone(),), context)
}
