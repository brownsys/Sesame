use rocket::State;
use std::sync::{Mutex, Arc};
use alohomora::{bbox::BBox, policy::NoPolicy};
use alohomora::rocket::{BBoxRedirect, BBoxTemplate, BBoxForm};
use alohomora::db::BBoxRow;
use alohomora::pure::{execute_pure, PrivacyPureRegion};
use alohomora_derive::{get, post};
use crate::backend::MySqlBackend;
use crate::common::*;
use crate::context::*;

#[get("/<gc_name>/<user_name>")]
pub(crate) fn try_show_gc(gc_name: BBox<String, NoPolicy>,
                          user_name: BBox<String, NoPolicy>,
                          backend: &State<Arc<Mutex<MySqlBackend>>>,
                          context: YouChatContext
) -> AnyBBoxResponse {
    //validate user first
    match validate_user_permissions(gc_name.clone(), user_name.clone(), backend, context.clone()) {
        PermissionType::Admin => 
            AnyBBoxResponse::Template(show_gc(gc_name, user_name, backend, context, PermissionType::Admin)),
        PermissionType::Member => 
            AnyBBoxResponse::Template(show_gc(gc_name, user_name, backend, context, PermissionType::Member)),
        PermissionType::None => {
            AnyBBoxResponse::Redirect(BBoxRedirect::to("/login", (), context))
        },
    }
}

fn show_gc(gc_name: BBox<String, NoPolicy>, 
            user_name: BBox<String, NoPolicy>, 
            backend: &State<Arc<Mutex<MySqlBackend>>>, 
            context: YouChatContext, 
            perm: PermissionType
) -> BBoxTemplate {
    let recieved = {
        // query database for recieved group chats
        let mut bg = backend.lock().unwrap();
        let received = (*bg).handle
            .prep_exec_iter(
                "SELECT * from chats WHERE group_chat = ? ORDER BY  time", 
                vec![gc_name.clone()], 
                context.clone())
            .unwrap()
            .enumerate()
            .map(|(index, row_result ): (usize, mysql::Result<BBoxRow>) | -> Chat {
                Chat::new(row_result.unwrap(), index)
            })
            .collect::<Vec<_>>(); 
        received
    };
    
    let ctx: GroupChatContext = GroupChatContext {
        group_name: gc_name,
        user_name: user_name,
        chats: recieved,
        admin: (perm == PermissionType::Admin),
    };

    BBoxTemplate::render("group_chat", &ctx, context)
}

fn validate_user_permissions(gc_name: BBox<String, NoPolicy>, 
                            user_name: BBox<String, NoPolicy>, 
                            backend: &State<Arc<Mutex<MySqlBackend>>>, 
                            context: YouChatContext
) -> PermissionType {
    let groups: Vec<Group> = backend.lock().unwrap().handle 
        .prep_exec_iter(
            "SELECT * FROM group_chats WHERE group_name = ?", 
            vec![gc_name.clone()], 
            context.clone())
        .unwrap()
        .map(|row_result: mysql::Result<BBoxRow> | -> Group {
            let row = row_result.unwrap(); 
            Group::from_row(row)
        })
        .collect::<Vec<_>>(); 
    
    let group = groups.get(0); //should only have one gc of name
    if group.is_none(){
        return PermissionType::None;
    } 
    let group = group.unwrap(); 

    let mut bg = backend.lock().unwrap();
    let user_codes: Vec<UserCode> = (*bg).handle
        .prep_exec_iter(
            "SELECT * FROM users_group WHERE user_name = ?", 
            vec![user_name.clone()], 
            context.clone())
        .unwrap()
        .map(|row_result: mysql::Result<BBoxRow> | -> UserCode {
            let row = row_result.unwrap(); 
            UserCode::from_row(row)
        })
        .collect::<Vec<_>>();

    // check to see if the user is an admin
    if *(user_name.discard_box()) == *(group.clone().admin.discard_box()) {
        return PermissionType::Admin;
    }

    // check to make sure that user has valid credential to be in group
    let group_code = group.clone().access_code.discard_box();
    for code in user_codes {
        if code.access_code.discard_box() == group_code {
            return PermissionType::Member;
        }
    }
    
    return PermissionType::None;
}

#[post("/<gc_name>/<user_name>/<index>/delete")]
pub(crate) fn try_delete(gc_name: BBox<String, NoPolicy>, 
                        user_name: BBox<String, NoPolicy>, 
                        index: BBox<usize, NoPolicy>, 
                        backend: &State<Arc<Mutex<MySqlBackend>>>, 
                        context: YouChatContext
) -> BBoxRedirect {
    // validate user permissions
    match validate_user_permissions(gc_name.clone(), user_name.clone(), backend, context.clone()) {
        PermissionType::Admin => delete(gc_name, user_name, index, backend, context.clone()),
        PermissionType::Member => {
            // if member, check if they're the author of the chat
            let to_delete = {
                let chats: Vec<Chat> = backend.lock().unwrap().handle
                    .prep_exec_iter(
                        "SELECT * from chats WHERE group_chat = ? ORDER BY time", 
                        vec![gc_name.clone()], 
                        context.clone())
                    .unwrap()
                    .map(|row_result: mysql::Result<BBoxRow> | -> Chat {
                        let row = row_result.unwrap(); 
                        Chat::from_row(row)
                    })
                    .collect::<Vec<_>>();
                
                chats[index.clone().discard_box()].clone()
            };

            // Ensure they can delete.
            let can_delete = execute_pure(
                (user_name.clone(), to_delete.sender.clone()),
                PrivacyPureRegion::new(|(user_name, sender): (String, String)| {
                    if user_name == sender {
                        Ok(())
                    } else {
                        Err("Permission denied")
                    }
                }),
            );
            
            if can_delete.unwrap().transpose().is_ok() {
                delete(gc_name, user_name, index, backend, context.clone())
            } else {
                BBoxRedirect::to("/chat/{}/{}", (&gc_name, &user_name), context.clone())
            }
        },
        PermissionType::None => { 
            BBoxRedirect::to("/login", (), context) 
        },
    }
}

pub(crate) fn delete(gc_name: BBox<String, NoPolicy>, 
                    user_name: BBox<String, NoPolicy>, 
                    index: BBox<usize, NoPolicy>, 
                    backend: &State<Arc<Mutex<MySqlBackend>>>, 
                    context: YouChatContext
) -> BBoxRedirect {
    // figure out which chat we want to delete
    let to_delete = {
        let res: Vec<Chat> = backend.lock().unwrap().handle
            .prep_exec_iter(
                "SELECT * from chats WHERE group_chat = ? ORDER BY time", 
                vec![gc_name.clone()], 
                context.clone())
            .unwrap()
            .map(|row_result: mysql::Result<BBoxRow> | -> Chat {
                let row = row_result.unwrap(); 
                Chat::from_row(row)
            })
            .collect::<Vec<_>>(); 
        
        res[index.discard_box()].clone()
    };

    // send query to delete that chat
    let _ = backend.lock().unwrap().handle.prep_exec_drop(
        "DELETE FROM chats WHERE group_chat = ? AND sender = ? AND recipient = ? AND content = ?", 
        (gc_name.clone(), to_delete.sender, to_delete.recipient, to_delete.content), 
        context.clone()).unwrap();

    BBoxRedirect::to("/chat/{}/{}", (&gc_name, &user_name), context)
}

#[post("/<gc_name>/<user_name>/send", data="<data>")]
pub(crate) fn send(gc_name: BBox<String, NoPolicy>, 
                    user_name: BBox<String, NoPolicy>, 
                    data: BBoxForm<MessageRequest>,
                    backend: &State<Arc<Mutex<MySqlBackend>>>,
                    context: YouChatContext
) -> BBoxRedirect {
    // get timestamp of send
    let time = timestamp();

    // assemble values to insert
    let vals = (
        data.recipient.clone(),
        user_name.clone(),
        data.content.clone(),
        time.clone(),
        gc_name.clone()
    );

    // make insert query
    let mut bg = backend.lock().unwrap();
    let _ = (*bg).handle
        .prep_exec_drop(
            "INSERT INTO chats VALUES (?,?,?,?,?)", 
            vals, 
            context.clone())
        .unwrap();

    // redirect back to groupchat page
    BBoxRedirect::to("/chat/{}/{}", (&gc_name.clone(), &user_name.clone()), context)
}