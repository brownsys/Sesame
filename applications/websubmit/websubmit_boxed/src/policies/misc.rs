use alohomora::context::UnprotectedContext;
use alohomora::policy::{
    schema_policy, AnyPolicy, FrontendPolicy, Policy, PolicyAnd, Reason, SchemaPolicy,
};
use mysql::Value;
use rocket::http::Cookie;
use rocket::Request;

#[derive(Clone)]
#[schema_policy(table = "users", column = 1)]
pub struct QueryableOnly {}

// Content of apikey column can only be accessed by:
//   1. SELECT query
impl Policy for QueryableOnly {
    fn name(&self) -> String {
        "QueryableOnly".to_string()
    }

    fn check(&self, _context: &UnprotectedContext, reason: Reason) -> bool {
        match reason {
            Reason::DB(query, _) => query.starts_with("SELECT"),
            _ => false,
        }
    }

    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        if other.is::<QueryableOnly>() {
            // Policies are combinable
            Ok(other)
        } else {
            //Policies must be stacked
            Ok(AnyPolicy::new(PolicyAnd::new(self.clone(), other)))
        }
    }

    fn join_logic(&self, _other: Self) -> Result<Self, ()> {
        Ok(QueryableOnly {})
    }
}

impl FrontendPolicy for QueryableOnly {
    fn from_request<'a, 'r>(_request: &'a Request<'r>) -> Self {
        QueryableOnly {}
    }
    fn from_cookie<'a, 'r>(
        _name: &str,
        _cookie: &'a Cookie<'static>,
        _request: &'a Request<'r>,
    ) -> Self {
        QueryableOnly {}
    }
}

impl SchemaPolicy for QueryableOnly {
    fn from_row(_table: &str, _row: &Vec<Value>) -> Self
    where
        Self: Sized,
    {
        QueryableOnly {}
    }
}
