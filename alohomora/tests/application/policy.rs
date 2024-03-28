use std::collections::HashSet;
use cookie::Cookie;
use mysql::{from_value, Value};
use rocket::Request;
use alohomora::AlohomoraType;

use alohomora::context::UnprotectedContext;
use alohomora::policy::{AnyPolicy, FrontendPolicy, Policy, SchemaPolicy};
use alohomora_derive::schema_policy;

use crate::application::context::ContextData;

#[derive(Clone)]
#[schema_policy(table = "grades", column = 2)]
pub struct ACLPolicy {
    pub users: HashSet<String>,
}
impl Policy for ACLPolicy {
    fn name(&self) -> String {
        String::from("ACLPolicy")
    }
    fn check(&self, context: &UnprotectedContext) -> bool {
        type ContextDataOut = <ContextData as AlohomoraType>::Out;
        let r: &ContextDataOut = context.downcast_ref().unwrap();
        match r {
            None => false,
            Some(user) => self.users.contains(user),
        }
    }
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> where Self: Sized {
        todo!()
    }
}
impl SchemaPolicy for ACLPolicy {
    fn from_row(row: &Vec<Value>) -> Self where Self: Sized {
        ACLPolicy {
            users: HashSet::from([
                String::from("admin"),
                from_value(row[1].clone()),
            ]),
        }
    }
}

#[derive(Clone)]
pub struct InternalPolicy {}
impl Policy for InternalPolicy {
    fn name(&self) -> String {
        String::from("InternalPolicy")
    }
    fn check(&self, _: &UnprotectedContext) -> bool { false }
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> where Self: Sized {
        todo!()
    }
}
impl FrontendPolicy for InternalPolicy {
    fn from_request(_request: &'_ Request<'_>) -> Self {
        InternalPolicy {}
    }
    fn from_cookie<'a, 'r>(_name: &str, _cookie: &'a Cookie<'static>, _request: &'a Request<'r>) -> Self {
        InternalPolicy {}
    }
}

#[derive(Clone)]
pub struct WritePolicy {}
impl Policy for WritePolicy {
    fn name(&self) -> String {
        String::from("WritePolicy")
    }
    fn check(&self, context: &UnprotectedContext) -> bool {
        type ContextDataOut = <ContextData as AlohomoraType>::Out;
        let r: &ContextDataOut = context.downcast_ref().unwrap();
        match r {
            None => false,
            Some(user) => user == "admin",
        }
    }
    fn join(&self, _other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, _other: Self) -> Result<Self, ()> where Self: Sized {
        todo!()
    }
}
impl FrontendPolicy for WritePolicy {
    fn from_request(_request: &'_ Request<'_>) -> Self {
        WritePolicy {}
    }
    fn from_cookie<'a, 'r>(_name: &str, _cookie: &'a Cookie<'static>, _request: &'a Request<'r>) -> Self {
        WritePolicy {}
    }
}