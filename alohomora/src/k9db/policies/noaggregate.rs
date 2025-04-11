use std::collections::{HashMap};

use mysql::Value;
use crate::context::UnprotectedContext;
use crate::k9db::policies::{add_k9db_policy, K9dbPolicy};
use crate::policy::{register, AnyPolicy, Policy, Reason};

#[derive(Clone)]
pub struct NoAggregate {
    ok: bool,
}

impl Policy for NoAggregate {
    fn name(&self) -> String {
        String::from("NoAggregate")
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
        self.ok
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        todo!()
    }
}

impl K9dbPolicy for NoAggregate {
    fn from_row(metadata: Vec<String>) -> Self {
        NoAggregate {
            ok: metadata[0] == "1",
        }
    }
    fn order_args(mut args: HashMap<String, String>) -> Vec<String> {
        assert_eq!(args.len(), 0);
        Vec::new()
    }
    fn only_k9db() -> bool {
        true
    }
}
#[register]
unsafe fn register_access_control() {
    add_k9db_policy::<NoAggregate>(String::from("NoAggregate"));
}
