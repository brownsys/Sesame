mod cookie;
mod data;
mod form;
mod request;
mod response;
mod rocket;
mod route;
mod template;

pub use crate::rocket::cookie::*;
pub use crate::rocket::data::*;
pub use crate::rocket::form::*;
pub use crate::rocket::request::*;
pub use crate::rocket::response::*;
pub use crate::rocket::rocket::*;
pub use crate::rocket::route::*;
pub use crate::rocket::template::*;

// TODO(babman): Later: Policy  ----> PolicyClause and PolicyType
//                      PolicyClause: atomic unit, cannot be seperated out, can be re-used, potentially can derive from DSL.
//                                    API: Check, Serialize + Deserialize, Equality testing.
//                      PolicyType: combination of PolicyClauses with ands and ors.
//                                  internally represent using CNF (or restrict &&+|| alternation)
//                                  can be pruned by removing irrelevant clauses automatically and manually
//                                  PolicyTypes should be assigned a unique name to simplify refering to them
//                                  consistently throughout the code, e.g. "Email" can be used to decorate schema and forms and other incoming sinks.
//                                  API: check, Serialize + Deserialize, (Semantic) equality, pruning, intersection (w/ automatic pruning?)
//                                       prunning likely requires some notion of ->

// TODO(babman): Difficulty with Policy Transformations: Policies are not know statically: so hard for developers to figure out what to combine.
//               How can we help here?