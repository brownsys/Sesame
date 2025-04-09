use std::collections::{HashMap};
use mysql::Value;
use crate::context::UnprotectedContext;
use crate::k9db::policies::{add_k9db_policy, K9dbPolicy};
use crate::policy::{register, AnyPolicy, Policy, Reason};

#[derive(Clone)]
pub struct Aggregate {
    k: usize,
    min_k: usize,
}
impl Policy for Aggregate {
    fn name(&self) -> String {
        String::from("Aggregate")
    }
    fn check(&self, _context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
        self.k >= self.min_k
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        todo!()
    }
}

impl K9dbPolicy for Aggregate {
    fn from_row(metadata: Vec<String>) -> Self {
        Aggregate {
            k: metadata[0].clone().parse().unwrap(),
            min_k: metadata[1].clone().parse().unwrap(),
        }
    }
    fn order_args(mut args: HashMap<String, String>) -> Vec<String> {
        vec![String::from("v{1}"), args.remove("min_k").unwrap(),
             args.remove("distinct").unwrap()]
    }
}

#[register]
unsafe fn register_access_control() {
    add_k9db_policy::<Aggregate>(String::from("Aggregate"));
}
