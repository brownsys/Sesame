use crate::policy::policies::ChatAccessPolicy;
use chrono::prelude::*;
use sesame::pcon::PCon;
use sesame::policy::{AnyPolicy, NoPolicy};
use sesame::verified::{execute_verified, VerifiedRegion};
use sesame_mysql::{from_value, PConRow};
use sesame_rocket::render::PConRender;
use sesame_rocket::rocket::FromPConForm;

pub struct FromRowError(pub PConRow);

pub trait FromBBoxRow {
    fn from_row(row: PConRow) -> Self
    where
        Self: Sized,
    {
        match Self::from_row_opt(row) {
            Ok(s) => s,
            Err(FromRowError(_)) => panic!(
                "Couldn't convert BBoxRow to value" // add better debugging here
            ),
        }
    }

    fn from_row_opt(row: PConRow) -> Result<Self, FromRowError>
    where
        Self: Sized;
}

impl FromBBoxRow for PConRow {
    fn from_row(row: PConRow) -> Self
    where
        Self: Sized,
    {
        row
    }

    fn from_row_opt(row: PConRow) -> Result<Self, FromRowError>
    where
        Self: Sized,
    {
        Ok(row)
    }
}

// Variable for different types of permissions
#[derive(PartialEq)]
pub enum PermissionType {
    Admin,
    Member,
    None,
}

#[derive(PConRender)]
pub struct Chat {
    pub recipient: PCon<String, ChatAccessPolicy>,
    pub sender: PCon<String, ChatAccessPolicy>,
    pub content: PCon<String, ChatAccessPolicy>,
    pub timestamp: PCon<String, AnyPolicy>,
    pub index: PCon<usize, NoPolicy>, // the order in which the chat will be displayed on page
                                      // (so we can know which chats to delete on click)
}

impl Chat {
    pub fn new(row: PConRow, index: usize) -> Self {
        let b: PCon<mysql::Value, AnyPolicy> = row.get(3).unwrap();
        let t: PCon<String, AnyPolicy> = execute_verified(
            b,
            VerifiedRegion::new(|val: mysql::Value| val.as_sql(true).replace("'", "")),
        )
        .unwrap();
        Chat {
            recipient: from_value(row.get(0).unwrap()).unwrap(),
            sender: from_value(row.get(1).unwrap()).unwrap(),
            content: from_value(row.get(2).unwrap()).unwrap(),
            timestamp: t,
            index: PCon::new(index, NoPolicy::new()),
        }
    }
}

impl FromBBoxRow for Chat {
    fn from_row(row: PConRow) -> Self {
        Chat::new(row, 0)
    }

    fn from_row_opt(row: PConRow) -> Result<Self, FromRowError> {
        Ok(Chat::new(row, 0))
    }
}

#[derive(PConRender, Clone)]
pub struct Group {
    pub name: PCon<String, NoPolicy>,
    pub admin: PCon<String, NoPolicy>,
    pub access_code: PCon<String, NoPolicy>,
}

impl Group {
    pub fn new(row: PConRow) -> Self {
        Group {
            name: from_value(row.get(0).unwrap()).unwrap(),
            admin: from_value(row.get(1).unwrap()).unwrap(),
            access_code: from_value(row.get(2).unwrap()).unwrap(),
        }
    }
}

impl FromBBoxRow for Group {
    fn from_row(row: PConRow) -> Self {
        Group::new(row)
    }

    fn from_row_opt(row: PConRow) -> Result<Self, FromRowError> {
        Ok(Group::new(row))
    }
}

#[derive(PConRender)]
pub struct UserCode {
    pub user: PCon<String, NoPolicy>,
    pub access_code: PCon<String, NoPolicy>,
}

impl UserCode {
    pub fn new(row: PConRow) -> Self {
        UserCode {
            user: from_value(row.get(0).unwrap()).unwrap(),
            access_code: from_value(row.get(1).unwrap()).unwrap(),
        }
    }
}

impl FromBBoxRow for UserCode {
    fn from_row(row: PConRow) -> Self {
        UserCode::new(row)
    }

    fn from_row_opt(row: PConRow) -> Result<Self, FromRowError> {
        Ok(UserCode::new(row))
    }
}

// the context needed to render the chat webpage
// NOT TO BE CONFUSED with the context needed to open a ChatAccessPolicy BBox
#[derive(PConRender)]
pub struct ChatContext {
    pub name: PCon<String, NoPolicy>,
    pub sent_chats: Vec<Chat>,
    pub recieved_chats: Vec<Chat>,
    pub buggy: bool,
}

// the context needed to render the groupchat webpage
#[derive(PConRender)]
pub struct GroupChatContext {
    pub group_name: PCon<String, NoPolicy>,
    pub user_name: PCon<String, NoPolicy>,
    pub chats: Vec<Chat>,
    pub admin: bool,
}

#[derive(FromPConForm)]
pub struct MessageRequest {
    pub(crate) recipient: PCon<String, NoPolicy>,
    pub(crate) content: PCon<String, NoPolicy>,
}

// generates a string timestamp of the current time
pub(crate) fn timestamp() -> String {
    let timestamp: u64 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
        - (60 * 60 * 5);
    //^subtract 5 hours bc EST is UTC-5:00

    // Create a DateTime from the timestamp
    let datetime = DateTime::from_timestamp(timestamp as i64, 0).unwrap();

    // Format the datetime how you want
    let newdate = datetime.format("%Y-%m-%d %H:%M:%S");
    newdate.to_string()
}
