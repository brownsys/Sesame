use mysql::prelude::Queryable;

use alohomora::policy::{AnyPolicy, FromSchema, Policy, PolicyAnd, Reason, SchemaPolicy};
use alohomora::AlohomoraType;
use alohomora::context::UnprotectedContext;
use crate::context::*;

pub type ChatAccessPolicy = ChatAccessPolicy2;

#[derive(Clone, Debug)]
pub struct ChatUser(pub String, pub String, pub Option<String>);

impl ChatUser {
    fn is_sender(&self, ctx: &ContextDataOut) -> bool {
        ctx.user.clone().map_or(false, |name|{ name == self.0 })
    }

    fn is_reciever(&self, ctx: &ContextDataOut) -> bool {
        ctx.user.clone().map_or(false, |name|{ name == self.1 })
    }

    fn is_group_member(&self, ctx: &ContextDataOut) -> bool {
        let group_name = if self.2.is_some() { self.2.as_ref().unwrap() } else { return false; };
        let mut db = ctx.db.lock().unwrap();

        let groupchats_w_name: Vec<(String, String, String)> = 
            db.query(format!("SELECT * FROM group_chats WHERE group_name = \"{}\"", group_name)).unwrap();
        
        if let Some((_group_name, admin_name, group_code)) = groupchats_w_name.first() {
            if let Some(name) = ctx.user.as_ref() {
                if name == admin_name { return true; }
            }

            let codes: Vec<(String, String)> = db.query(format!("SELECT * FROM users_group WHERE user_name = \"{}\" AND access_code = \"{}\"",
                    ctx.user.as_ref().unwrap(), group_code)).unwrap();

            codes.len() >= 1
        } else { false }
    }
}

alohomora_policy::access_control_policy!(ChatAccessPolicy2, 
    ContextData,
    ChatUser,
    [is_sender || is_reciever || is_group_member, alohomora_policy::anything!()]
    [alohomora_policy::never_leaked!()]);

impl FromSchema for ChatUser {
    fn from_row(_: &str, row: &Vec<mysql::Value>) -> Self
        where Self: Sized {
        ChatUser(
            mysql::from_value(row[0].clone()), 
            mysql::from_value(row[1].clone()), 
            mysql::from_value(row[4].clone())
        )
    }
}