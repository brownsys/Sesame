use crate::policy::{Policy, AnyPolicy};

#[derive(Default, serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct ExamplePolicy {
    pub state: i32,
}

impl Policy for ExamplePolicy {
    fn name(&self) -> String {
        "".to_string()
    }
    fn check(
        &self,
        context: &crate::context::UnprotectedContext,
        reason: crate::policy::Reason<'_>,
    ) -> bool {
        true
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()>
    where
        Self: Sized,
    {
        todo!()
    }
}
