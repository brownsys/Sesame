use alohomora::SesameType;
use cookie::Cookie;
use mysql::{from_value, Value};
use rocket::Request;
use std::collections::HashSet;

use alohomora::context::UnprotectedContext;
use alohomora::policy::{FrontendPolicy, Join, Policy, Reason, SchemaPolicy, SimplePolicy};
use alohomora_derive::schema_policy;

use crate::application::context::ContextData;

#[derive(Clone)]
#[schema_policy(table = "grades", column = 2)]
pub struct ACLPolicy {
    pub users: HashSet<String>,
}
impl SimplePolicy for ACLPolicy {
    fn simple_name(&self) -> String {
        String::from("ACLPolicy")
    }
    fn simple_check(&self, context: &UnprotectedContext, _: Reason) -> bool {
        type ContextDataOut = <ContextData as SesameType>::Out;
        let r: &ContextDataOut = context.downcast_ref().unwrap();
        match r {
            None => false,
            Some(user) => self.users.contains(user),
        }
    }
    fn simple_join_direct(&mut self, other: &mut Self) {
        self.users = self
            .users
            .intersection(&other.users)
            .map(Clone::clone)
            .collect();
    }
}
impl SchemaPolicy for ACLPolicy {
    fn from_row(_table_name: &str, row: &Vec<Value>) -> Self
    where
        Self: Sized,
    {
        ACLPolicy {
            users: HashSet::from([String::from("admin"), from_value(row[1].clone())]),
        }
    }
}

#[derive(Clone)]
pub struct AuthenticationCookiePolicy {}
impl Join for AuthenticationCookiePolicy {}
impl Policy for AuthenticationCookiePolicy {
    fn name(&self) -> String {
        String::from("InternalPolicy")
    }
    fn check(&self, _: &UnprotectedContext, reason: Reason) -> bool {
        match reason {
            Reason::Cookie(name) => name == "user",
            Reason::DB(query, _) => query.starts_with("SELECT"),
            _ => false,
        }
    }
}
impl FrontendPolicy for AuthenticationCookiePolicy {
    fn from_request(_request: &'_ Request<'_>) -> Self {
        AuthenticationCookiePolicy {}
    }
    fn from_cookie<'a, 'r>(
        _name: &str,
        _cookie: &'a Cookie<'static>,
        _request: &'a Request<'r>,
    ) -> Self {
        AuthenticationCookiePolicy {}
    }
}

#[derive(Clone)]
pub struct WritePolicy {}
impl Join for WritePolicy {}
impl Policy for WritePolicy {
    fn name(&self) -> String {
        String::from("WritePolicy")
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason) -> bool {
        match reason {
            Reason::DB(stmt, _) => {
                if stmt.starts_with("INSERT") {
                    type ContextDataOut = <ContextData as SesameType>::Out;
                    let r: &ContextDataOut = context.downcast_ref().unwrap();
                    match r {
                        None => false,
                        Some(user) => user == "admin",
                    }
                } else {
                    true
                }
            }
            _ => false,
        }
    }
}
impl FrontendPolicy for WritePolicy {
    fn from_request(_request: &'_ Request<'_>) -> Self {
        WritePolicy {}
    }
    fn from_cookie<'a, 'r>(
        _name: &str,
        _cookie: &'a Cookie<'static>,
        _request: &'a Request<'r>,
    ) -> Self {
        WritePolicy {}
    }
}
