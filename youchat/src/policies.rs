use mysql::prelude::Queryable;

use alohomora::policy::{AnyPolicy, Policy, PolicyAnd, SchemaPolicy, Reason};
use alohomora::AlohomoraType;
use alohomora_derive::schema_policy;
use alohomora::context::UnprotectedContext;
use alohomora::db::BBoxParam;
use crate::context::*;

// access control policy for the chats database
// #[schema_policy(table = "chats", column = 0)]
// #[schema_policy(table = "chats", column = 1)]
// #[schema_policy(table = "chats", column = 2)]
// // #[schema_policy(table = "chats", column = 3)] // is bboxing the time really neccessary
// #[schema_policy(table = "chats", column = 4)] 

// policy for accessing chats
// only the sender, recipient, or members of the groupchat should have access
#[derive(Clone)]
pub struct ChatAccessPolicy {
    sender: Option<String>,
    recipient: Option<String>,
    groupchat: Option<String>
}

impl ChatAccessPolicy {
    pub fn new(sender: Option<String>, 
                recipient: Option<String>, 
                groupchat: Option<String>)
                -> ChatAccessPolicy{
        ChatAccessPolicy{
            sender: sender,
            recipient: recipient,
            groupchat: groupchat
        }
    }
}

impl Policy for ChatAccessPolicy {
    fn name(&self) -> String {
         format!("ChatAccessPolicy")
    }

    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        if let Reason::DB(_, _) = reason {
            return true;
        }
    
        type ContextDataOut = <ContextData as AlohomoraType>::Out;
        let context: &ContextDataOut = context.downcast_ref().unwrap();

        let user: &Option<String> = &context.user;

        // check sender, receiver
        if let Some(name) = user {
            // check if we're the sender or the reciever
            if self.sender.is_some() && self.sender.as_ref().unwrap() == name {
                return true;
            }
            if self.recipient.is_some() && self.recipient.as_ref().unwrap() == name {
                return true;
            }
    
            // check if we're a group admin or group member
            if let Some(groupchat) = &self.groupchat {
                // get group
                let mut db = context.db.lock().unwrap();
                let mut group_res = db.exec_iter(
                        "SELECT * FROM group_chats WHERE group_name = ?",
                        (groupchat.to_owned(),),
                ).unwrap();

                let (group_admin, group_code): (String, String) = match group_res.next() {
                    None => { return false },
                    Some(Err(_)) => { return false },
                    Some(Ok(group)) => (
                        mysql::from_value(group.get(1).unwrap()),
                        mysql::from_value(group.get(2).unwrap()),
                    ),
                };
                drop(group_res);
                drop(db);

                // check if we're admin
                if name == &group_admin {
                    return true;
                }
    
                // check if the user has a code that match the group's
                let mut db = context.db.lock().unwrap();
                let mut codes_res = db.exec_iter(
                    "SELECT * FROM users_group WHERE user_name = ? AND access_code = ?",
                    (name.clone(), group_code),
                ).unwrap();

                return codes_res.next().is_some();
            }
        }
        // otherwise, we shouldn't be able to see the chat
        false
    }

    fn join(&self, other: alohomora::policy::AnyPolicy) -> Result<AnyPolicy, ()> {
        if other.is::<ChatAccessPolicy>() { //Policies are combinable
            let other = other.specialize::<ChatAccessPolicy>().unwrap();
            Ok(AnyPolicy::new(self.join_logic(other)?))
        } else {                    //Policies must be stacked
            Ok(AnyPolicy::new(
                PolicyAnd::new(
                    AnyPolicy::new(self.clone()),
                    other)))
        }
    }

    fn join_logic(&self, p2: Self) -> Result<Self, ()> {
        let mut comp_sender: Option<String> = None;
        let mut comp_recipient: Option<String> = None;
        let mut comp_groupchat: Option<String> = None;

        if self.sender.eq(&p2.sender) { comp_sender = self.sender.clone(); }
        if self.recipient.eq(&p2.recipient) { comp_recipient = self.recipient.clone(); }
        if self.groupchat.eq(&p2.groupchat) { comp_groupchat = self.groupchat.clone(); }

        Ok(ChatAccessPolicy{
            sender: comp_sender,
            recipient: comp_recipient,
            groupchat: comp_groupchat
        })
    }
}

impl SchemaPolicy for ChatAccessPolicy {
    fn from_row(table_name: &str, row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized,
    {
        ChatAccessPolicy{
            sender: mysql::from_value(row[0].clone()),
            recipient: mysql::from_value(row[1].clone()),
            groupchat: mysql::from_value(row[4].clone()),
        }
    }
}
