use crate::backend::MySqlBackend;
use crate::policy::context::*;
use crate::routes::common::*;
use rocket::State;
use sesame::error::SesameResult;
use sesame::pcon::PCon;
use sesame::policy::{AnyPolicy, NoPolicy};
use sesame::verified::{execute_verified, VerifiedRegion};
use sesame_mysql::PConRow;
use sesame_rocket::error::SesameRenderResult;
use sesame_rocket::rocket::{get, post, PConForm, PConRedirect, PConResponseEnum, PConTemplate};
use std::sync::{Arc, Mutex};

#[get("/<gc_name>/<user_name>")]
pub(crate) fn try_show_gc(
    gc_name: PCon<String, NoPolicy>,
    user_name: PCon<String, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: YouChatContext,
) -> PConResponseEnum {
    //validate user first
    match validate_user_permissions(gc_name.clone(), user_name.clone(), backend, context.clone()) {
        PermissionType::Admin => {
            show_gc(gc_name, user_name, backend, context, PermissionType::Admin).into()
        }
        PermissionType::Member => {
            show_gc(gc_name, user_name, backend, context, PermissionType::Member).into()
        }
        PermissionType::None => PConRedirect::to("/login", (), context).into(),
    }
}

fn show_gc(
    gc_name: PCon<String, NoPolicy>,
    user_name: PCon<String, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: YouChatContext,
    perm: PermissionType,
) -> SesameRenderResult<PConTemplate> {
    let recieved = {
        // query database for recieved group chats
        let mut bg = backend.lock().unwrap();
        let received = (*bg)
            .handle
            .prep_exec_iter(
                "SELECT * from chats WHERE group_chat = ? ORDER BY  time",
                vec![gc_name.clone()],
                context.clone(),
            )
            .unwrap()
            .enumerate()
            .map(
                |(index, row_result): (usize, mysql::Result<PConRow>)| -> Chat {
                    Chat::new(row_result.unwrap(), index)
                },
            )
            .collect::<Vec<_>>();
        received
    };

    let ctx: GroupChatContext = GroupChatContext {
        group_name: gc_name,
        user_name: user_name,
        chats: recieved,
        admin: (perm == PermissionType::Admin),
    };

    PConTemplate::render("group_chat", &ctx, context)
}

fn validate_user_permissions(
    gc_name: PCon<String, NoPolicy>,
    user_name: PCon<String, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: YouChatContext,
) -> PermissionType {
    let groups: Vec<Group> = backend
        .lock()
        .unwrap()
        .handle
        .prep_exec_iter(
            "SELECT * FROM group_chats WHERE group_name = ?",
            vec![gc_name.clone()],
            context.clone(),
        )
        .unwrap()
        .map(|row_result: mysql::Result<PConRow>| -> Group {
            let row = row_result.unwrap();
            Group::from_row(row)
        })
        .collect::<Vec<_>>();

    let group = groups.get(0); //should only have one gc of name
    if group.is_none() {
        return PermissionType::None;
    }
    let group = group.unwrap();

    let mut bg = backend.lock().unwrap();
    let user_codes: Vec<UserCode> = (*bg)
        .handle
        .prep_exec_iter(
            "SELECT * FROM users_group WHERE user_name = ?",
            vec![user_name.clone()],
            context.clone(),
        )
        .unwrap()
        .map(|row_result: mysql::Result<PConRow>| -> UserCode {
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
pub(crate) fn try_delete(
    gc_name: PCon<String, NoPolicy>,
    user_name: PCon<String, NoPolicy>,
    index: PCon<usize, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: YouChatContext,
) -> SesameResult<PConRedirect> {
    // validate user permissions
    match validate_user_permissions(gc_name.clone(), user_name.clone(), backend, context.clone()) {
        PermissionType::Admin => delete(gc_name, user_name, index, backend, context.clone()),
        PermissionType::Member => {
            // if member, check if they're the author of the chat
            let to_delete = {
                let mut chats: Vec<Chat> = backend
                    .lock()
                    .unwrap()
                    .handle
                    .prep_exec_iter(
                        "SELECT * from chats WHERE group_chat = ? ORDER BY time",
                        vec![gc_name.clone()],
                        context.clone(),
                    )
                    .unwrap()
                    .map(|row_result: mysql::Result<PConRow>| -> Chat {
                        let row = row_result.unwrap();
                        Chat::from_row(row)
                    })
                    .collect::<Vec<_>>();

                chats.remove(index.clone().discard_box())
            };

            // Ensure they can delete.
            let can_delete: Result<PCon<_, AnyPolicy>, ()> = execute_verified(
                (user_name.clone(), to_delete.sender.clone()),
                VerifiedRegion::new(|(user_name, sender): (String, String)| {
                    if user_name == sender {
                        Ok(())
                    } else {
                        Err("Permission denied")
                    }
                }),
            );

            if can_delete.unwrap().fold_in().is_ok() {
                delete(gc_name, user_name, index, backend, context.clone())
            } else {
                PConRedirect::to("/chat/{}/{}", (&gc_name, &user_name), context.clone())
            }
        }
        PermissionType::None => PConRedirect::to("/login", (), context),
    }
}

pub(crate) fn delete(
    gc_name: PCon<String, NoPolicy>,
    user_name: PCon<String, NoPolicy>,
    index: PCon<usize, NoPolicy>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: YouChatContext,
) -> SesameResult<PConRedirect> {
    // figure out which chat we want to delete
    let to_delete = {
        let mut res: Vec<Chat> = backend
            .lock()
            .unwrap()
            .handle
            .prep_exec_iter(
                "SELECT * from chats WHERE group_chat = ? ORDER BY time",
                vec![gc_name.clone()],
                context.clone(),
            )
            .unwrap()
            .map(|row_result: mysql::Result<PConRow>| -> Chat {
                let row = row_result.unwrap();
                Chat::from_row(row)
            })
            .collect::<Vec<_>>();

        res.remove(index.discard_box())
    };

    // send query to delete that chat
    let _ = backend.lock().unwrap().handle.prep_exec_drop(
        "DELETE FROM chats WHERE group_chat = ? AND sender = ? AND recipient = ? AND content = ?", 
        (gc_name.clone(), to_delete.sender, to_delete.recipient, to_delete.content),
        context.clone()).unwrap();

    PConRedirect::to("/chat/{}/{}", (&gc_name, &user_name), context)
}

#[post("/<gc_name>/<user_name>/send", data = "<data>")]
pub(crate) fn send(
    gc_name: PCon<String, NoPolicy>,
    user_name: PCon<String, NoPolicy>,
    data: PConForm<MessageRequest>,
    backend: &State<Arc<Mutex<MySqlBackend>>>,
    context: YouChatContext,
) -> SesameResult<PConRedirect> {
    // get timestamp of send
    let time = timestamp();

    // assemble values to insert
    let vals = (
        data.recipient.clone(),
        user_name.clone(),
        data.content.clone(),
        time.clone(),
        gc_name.clone(),
    );

    // make insert query
    let mut bg = backend.lock().unwrap();
    let _ = (*bg)
        .handle
        .prep_exec_drop(
            "INSERT INTO chats VALUES (?,?,?,?,?)",
            vals,
            context.clone(),
        )
        .unwrap();

    // redirect back to groupchat page
    PConRedirect::to(
        "/chat/{}/{}",
        (&gc_name.clone(), &user_name.clone()),
        context,
    )
}
