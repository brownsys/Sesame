
use core::panic;

use alohomora::{orm::ORMPolicy, policy::{AnyPolicy, FrontendPolicy, NoPolicy, Policy, PolicyAnd}, testing::TestContextData, AlohomoraType};
use sea_orm::{ConnectionTrait, DatabaseConnection, Statement};
use serde::Serialize;

#[derive(Clone, Serialize, Debug, PartialEq)]
pub struct CandidateDataPolicy {
    // only set for data coming from client POST
    session_id: Option<String>, 
    
    // only set for data coming from DB
    candidate_id: Option<i32>,    // (candidate table)
    application_id: Option<i32>,  // (application table)
}

// (->) candidate data can enter the system
//     a. as post data (we have session_id cookie) <- FrontendPolicy
//     b. as query from DB (we have candidate_id)  <- ORMPolicy

// (<-) candidate data can be leaked
//     a. as rendering (we have session_id, pk?)
//     b. into pcr (we don't care to validate)
//     c. into db  (^^)

impl CandidateDataPolicy {
    pub fn new(candidate_id: Option<i32>) -> Self {
        CandidateDataPolicy{ 
            session_id: None,
            candidate_id,
            application_id: None,
        }
    }
}

impl Default for CandidateDataPolicy {
    fn default() -> Self {
        // println!("defaulting!!");
        CandidateDataPolicy { session_id: None, candidate_id: None, application_id: None }
    }
}

impl Policy for CandidateDataPolicy {
    fn name(&self) -> String {
        match self.candidate_id {
            Some(id) => format!("Candidate Data Policy (id: {id})"),
            None => format!("Candidate Data Policy (only accessible by admins)"),
        }
    }

    // right client (cand_id) render -> ok
    // any admin render -> ok
    // right client (in session) db -> ok
    // custom region -> okay
    // EVERYTHING ELSE -> nuh uh

    fn check(&self, context: &alohomora::context::UnprotectedContext, reason: alohomora::policy::Reason<'_>) -> bool {
        todo!()
    }

    fn join(&self, other: alohomora::policy::AnyPolicy) -> Result<alohomora::policy::AnyPolicy, ()> {
        if other.is::<CandidateDataPolicy>() {
            let other = other.specialize().unwrap();
            return Ok(AnyPolicy::new(self.join_logic(other)?));
        } else {
            // println!("data stacking polciies w/ other {:?}", other);
            if other == AnyPolicy::new(NoPolicy::new()){ // TODO: why do I need this??
                return Ok(AnyPolicy::new(self.clone()));
            }
            return Ok(AnyPolicy::new(PolicyAnd::new(
                AnyPolicy::new(self.clone()), 
                other)
            ));
        }
    }

    fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized {
        let (mut candidate_id, mut session_id, mut application_id) = (None, None, None);
        if self.candidate_id == other.candidate_id {
            // if they have the same id, keep it
            candidate_id = self.candidate_id;
        }
        if self.session_id == other.session_id {
            session_id = self.session_id.clone();
        }
        if self.application_id == other.application_id {
            application_id = self.application_id.clone();
        }
        Ok(CandidateDataPolicy{
            candidate_id,
            session_id,
            application_id
        })
    }
}

impl ORMPolicy for CandidateDataPolicy {
    fn from_result(result: &sea_orm::prelude::QueryResult) -> Self {
        todo!()
    }

    fn empty() -> Self where Self: Sized {
        CandidateDataPolicy{
            candidate_id: None,
            application_id: None,
            session_id: None,
        }
    }
}

impl FrontendPolicy for CandidateDataPolicy {
    fn from_cookie<'a, 'r>(
            name: &str,
            cookie: &'a rocket::http::Cookie<'static>,
            request: &'a rocket::Request<'r>) -> Self where Self: Sized {
        Self::from_request(request)
    }

    fn from_request<'a, 'r>(request: &'a rocket::Request<'r>) -> Self
            where
                Self: Sized {
        todo!()
    }
}
