use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use mysql::Value;
use crate::context::UnprotectedContext;
use crate::k9db::context::UnprotectedK9dbContextData;
use crate::k9db::policies::{K9dbPolicy, register, add_k9db_policy};
use crate::policy::{AnyPolicy, Policy, Reason};

#[derive(Clone)]
pub struct AccessControl {
    users: HashSet<String>,
}

impl Policy for AccessControl {
    fn name(&self) -> String {
        String::from("AccessControl")
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
        let context = context.downcast_ref::<UnprotectedK9dbContextData>().unwrap();
        match &context.user {
            None => false,
            Some(user) => self.users.contains(user),
        }
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        todo!()
    }
}

impl K9dbPolicy for AccessControl {
    fn from_row(metadata: Vec<String>) -> Self {
        AccessControl {
            users: HashSet::from_iter(metadata.into_iter()),
        }
    }
    fn order_args(mut args: HashMap<String, String>) -> Vec<String> {
        (0..args.len()).map(|i| args.remove(&i.to_string()).unwrap()).collect()
    }
}

#[register]
unsafe fn register_access_control() {
    add_k9db_policy::<AccessControl>(String::from("AccessControl"));
}
