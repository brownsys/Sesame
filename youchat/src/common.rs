extern crate chrono;
//use std::any::Any;

use chrono::prelude::*;
use alohomora::bbox::BBox;
use alohomora::db::{BBoxRow, from_value, BBoxFromValue};
use alohomora::rocket::BBoxResponseEnum;
use alohomora_derive::{BBoxRender, FromBBoxForm};
use alohomora::policy::{AnyPolicy, NoPolicy}; 
use alohomora::pure::{PrivacyPureRegion, execute_pure};
use crate::policies::ChatAccessPolicy;

// Response enum that can be either templates or redirects
pub type AnyBBoxResponse = BBoxResponseEnum; 
pub struct FromRowError(pub BBoxRow);

pub trait FromBBoxRow {
    fn from_row(row: BBoxRow) -> Self
    where
        Self: Sized,
    {
        match Self::from_row_opt(row) {
            Ok(s) => s,
            Err(FromRowError(_)) => panic!(
                "Couldn't convert BBoxRow to value"
                // add better debugging here
            ),
        }
    }

    fn from_row_opt(row: BBoxRow) -> Result<Self, FromRowError>
    where
        Self: Sized;
}

impl FromBBoxRow for BBoxRow {
    fn from_row(row: BBoxRow) -> Self
    where
        Self: Sized {
        row
    }

    fn from_row_opt(row: BBoxRow) -> Result<Self, FromRowError> 
    where
        Self: Sized {
        Ok(row)
    }
}

impl<T> FromBBoxRow for BBox<T, AnyPolicy>
where T: BBoxFromValue {
    fn from_row_opt(row: BBoxRow) -> Result<BBox<T, AnyPolicy>, FromRowError> {
        if  row.clone().unwrap().len() == 1 {
            let val: Result<BBox<T, AnyPolicy>, String> = from_value(row.clone().get(0).unwrap().clone());
            match val {
                Err(e) => {
                    //panic!("Couldn't convert BBoxRow to value");
                    Err(FromRowError(row))
                },
                Ok(v ) => Ok(v)
            }
        } else {
            Err(FromRowError(row))
        }
    }
}

// Variable for different types of permissions
#[derive(PartialEq)]
pub enum PermissionType {
    Admin,
    Member,
    None
}

#[derive(BBoxRender, Clone)]
pub struct Chat {
    pub recipient: BBox<String, ChatAccessPolicy>,
    pub sender: BBox<String, ChatAccessPolicy>,
    pub content: BBox<String, ChatAccessPolicy>,
    pub timestamp: BBox<String, AnyPolicy>,
    pub index: BBox<usize, NoPolicy>, // the order in which the chat will be displayed on page
                                      // (so we can know which chats to delete on click)
}

impl Chat {
    pub fn new(row: BBoxRow, index: usize) -> Self {
        let b: BBox<mysql::Value, AnyPolicy> = row.get(3).unwrap();
        let t: BBox<String, AnyPolicy> = execute_pure(
                                    b.clone(), 
                                    PrivacyPureRegion::new(|val: mysql::Value| 
                                                { val.as_sql(true).replace("'", "")}))
                                        .unwrap(); 
        Chat {
            recipient: from_value(row.get(0).unwrap()).unwrap(),
            sender: from_value(row.get(1).unwrap()).unwrap(),
            content: from_value(row.get(2).unwrap()).unwrap(),
            timestamp: t,
            index: BBox::new(index, NoPolicy::new()),
        }
    } 
}

impl FromBBoxRow for Chat {
    fn from_row(row: BBoxRow) -> Self { Chat::new(row, 0) }

    fn from_row_opt(row: BBoxRow) -> Result<Self, FromRowError> {
        Ok(Chat::new(row, 0))
    }
}

#[derive(BBoxRender, Clone)]
pub struct Group {
    pub name: BBox<String, NoPolicy>,
    pub admin: BBox<String, NoPolicy>,
    pub access_code: BBox<String, NoPolicy>,
}

impl Group {
    pub fn new(row: BBoxRow) -> Self {
        Group{
            name: from_value(row.get(0).unwrap()).unwrap(),
            admin: from_value(row.get(1).unwrap()).unwrap(),
            access_code: from_value(row.get(2).unwrap()).unwrap(),
        }
    } 
}

impl FromBBoxRow for Group {
    fn from_row(row: BBoxRow) -> Self { Group::new(row) }

    fn from_row_opt(row: BBoxRow) -> Result<Self, FromRowError> {
        Ok(Group::new(row))
    }
}

#[derive(BBoxRender)]
pub struct UserCode {
    pub user: BBox<String, NoPolicy>,
    pub access_code: BBox<String, NoPolicy>,
}

impl UserCode {
    pub fn new(row: BBoxRow) -> Self {
        UserCode{
            user: from_value(row.get(0).unwrap()).unwrap(),
            access_code: from_value(row.get(1).unwrap()).unwrap(),
        }
    }
}

impl FromBBoxRow for UserCode {
    fn from_row(row: BBoxRow) -> Self { UserCode::new(row) }

    fn from_row_opt(row: BBoxRow) -> Result<Self, FromRowError> { 
        Ok(UserCode::new(row))
    }
}

// the context needed to render the chat webpage
// NOT TO BE CONFUSED with the context needed to open a ChatAccessPolicy BBox
#[derive(BBoxRender)]
pub struct ChatContext {
    pub name: BBox<String, NoPolicy>,
    pub sent_chats: Vec<Chat>,
    pub recieved_chats: Vec<Chat>,
    pub buggy: bool,
}

// the context needed to render the groupchat webpage
#[derive(BBoxRender)]
pub struct GroupChatContext {
    pub group_name: BBox<String, NoPolicy>,
    pub user_name: BBox<String, NoPolicy>,
    pub chats: Vec<Chat>,
    pub admin: bool,
}

#[derive(FromBBoxForm)]
pub struct MessageRequest {
    pub(crate) recipient: BBox<String, NoPolicy>,
    pub(crate) content: BBox<String, NoPolicy>,
}

// generates a string timestamp of the current time
pub(crate) fn timestamp() -> String {
    let timestamp: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() - (60 * 60 * 5);
                    //^subtract 5 hours bc EST is UTC-5:00
    
    // Create a NaiveDateTime from the timestamp
    let naive = NaiveDateTime::from_timestamp_opt(timestamp as i64, 0).unwrap();
    
    // Create a normal DateTime from the NaiveDateTime
    let datetime: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive, Utc);
    
    // Format the datetime how you want
    let newdate = datetime.format("%Y-%m-%d %H:%M:%S");
    newdate.to_string()
}
