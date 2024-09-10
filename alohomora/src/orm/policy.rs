use sea_orm::QueryResult;

use crate::policy::{Policy, NoPolicy};

pub trait ORMPolicy : Policy {
    fn from_result(result: &QueryResult) -> Self
    where
        Self: Sized;

    fn empty() -> Self where Self: Sized;
}


impl ORMPolicy for NoPolicy {
    fn from_result(_result: &QueryResult) -> Self where Self: Sized {
        NoPolicy {}
    }
    fn empty() -> NoPolicy {
        NoPolicy {}
    }
}
