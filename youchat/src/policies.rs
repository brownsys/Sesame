use std::sync::{Arc, Mutex};

use alohomora::policy::{AnyPolicy, Policy, PolicyAnd, SchemaPolicy, Reason};
use alohomora::AlohomoraType;
use alohomora_derive::schema_policy;
use alohomora::context::UnprotectedContext;
use crate::backend::MySqlBackend;
use crate::context::*;
use crate::common::Group;

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
    #[allow(dead_code)]
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
        if self.groupchat.is_none() { 
            format!("ChatAccessPolicy--chat from {} to {}", 
                self.sender.clone().unwrap_or("[none]".to_string()), 
                self.recipient.clone().unwrap_or("[none]".to_string()))
        } else {
            format!("ChatAccessPolicy--chat from {} to {} (in {})", 
                self.sender.clone().unwrap_or("[none]".to_string()), 
                self.recipient.clone().unwrap_or("[none]".to_string()), 
                self.groupchat.clone().unwrap())
        }
    }

    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        if let Reason::DB(_) = reason {
            return true;
        }
    
        type ContextDataOut = <ContextData as AlohomoraType>::Out;
        let context: &ContextDataOut = context.downcast_ref().unwrap();

        let user: &Option<String> = &context.user;
        let db: &Arc<Mutex<MySqlBackend>> = &context.db;
        //let config: &Config = &context.config;

        // check sender, receiver
        if let Some(name) = user {   
            // check if we're the sender or the reciever
            match self.sender.clone() {
                Some(sender) => 
                    if sender == *name { return true; }
                None => ()
            }
            match self.recipient.clone() {
                Some(recipient) => 
                    if recipient == *name { return true; }
                None => ()
            }
    
        // check if we're a group admin or group member
            if let Some(groupchat) = self.groupchat.clone() {
                // get group
                let group = {
                    let mut bg = db.lock().unwrap();
                    let group_res: Vec<_> = (*bg).handle
                        .prep_exec_iter(
                            "SELECT * FROM group_chats WHERE group_name = ?", 
                            vec![groupchat.clone()], 
                            YouChatContext::empty())
                        .unwrap()
                        .collect();
                    
                    let mut group = None;
                    for g in group_res { //TODO (corinn) better way to handle this?
                        group = Some(Group::new(g.unwrap())); //or group_res[0]?
                        break;
                    }

                    // make sure it's a real group
                    if group.is_none() { return false; }
                    group
                }.unwrap();

                // check if we're admin
                if *name == *group.admin.discard_box() { return true; }
    
                // check if the user has a code that match the group's
                let mut bg = db.lock().unwrap();
                let codes_res = (*bg).handle.prep_exec_iter(
                    "SELECT * FROM users_group WHERE user_name = ? AND access_code = ?", 
                    vec![&name, &group.access_code.discard_box()],
                    YouChatContext::empty()
                ).unwrap(); 

                // if so, hooray!
                if codes_res.count() > 0 { return true; }
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
