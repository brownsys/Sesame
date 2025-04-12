use std::collections::{HashMap};
use mysql::Value;
use crate::context::UnprotectedContext;
use crate::k9db::context::UnprotectedK9dbContextData;
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
    fn check(&self, context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
        let context = context.downcast_ref::<UnprotectedK9dbContextData>().unwrap();
        match &context.purpose {
            None => false,
            Some(purpose) => self.consent && &self.purpose == purpose,
        }
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        if (other.is::<Self>()) {
            Ok(AnyPolicy::new(self.join_logic(other.specialize().unwrap())?))
        } else {
            todo!()
        }
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        assert_eq!(self.purpose, other.purpose);
        Ok(Self {
            consent: self.consent && other.consent,
            purpose: self.purpose.clone(),
        })
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
