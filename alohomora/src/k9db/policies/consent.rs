use std::collections::{HashMap};
use mysql::Value;
use crate::context::UnprotectedContext;
use crate::k9db::policies::{add_k9db_policy, K9dbPolicy};
use crate::policy::{register, AnyPolicy, Policy, Reason};

#[derive(Clone)]
pub struct Consent {
    consent: bool,
    purpose: String,
}

impl Policy for Consent {
    fn name(&self) -> String {
        String::from("Consent")
    }
    fn check(&self, _context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
        // TODO(babman): need to check purpose of the context.
        self.consent
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        todo!()
    }
}

impl K9dbPolicy for Consent {
    fn from_row(metadata: Vec<String>) -> Self {
        Consent {
            consent: metadata[0] == "1",
            purpose: metadata[1].clone(),
        }
    }
    fn order_args(mut args: HashMap<String, String>) -> Vec<String> {
        vec![args.remove("consent").unwrap(), args.remove("purpose").unwrap()]
    }
}
#[register]
unsafe fn register_access_control() {
    add_k9db_policy::<Consent>(String::from("Consent"));
}
