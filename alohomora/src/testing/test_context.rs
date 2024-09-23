use crate::{AlohomoraType, Unwrapper};
use crate::bbox::BBox;
use crate::context::{Context, UnprotectedContext};
use crate::policy::NoPolicy;
use crate::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};

#[derive(Clone)]
pub struct TestContextData<T: 'static>(BBox<T, NoPolicy>);

impl<T: Send + 'static> TestContextData<T> {
    pub fn new(t: T) -> Self {
        Self(BBox::new(t, NoPolicy {}))
    }
}

#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<T: Send + 'static> AlohomoraType for TestContextData<T> {
    type Out = T;
    type Policy = NoPolicy;

    fn inner_fold(self, unwrapper: &Unwrapper) -> Result<(Self::Out, Self::Policy), ()> {
        self.0.inner_fold(unwrapper)
    }
}

#[rocket::async_trait]
impl<'a, 'r, T: Send + 'static> FromBBoxRequest<'a, 'r> for TestContextData<T> {
    type BBoxError = ();
    async fn from_bbox_request(_request: BBoxRequest<'a, 'r>) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        todo!()
    }
}

impl<T: Send + 'static> Context<TestContextData<T>> {
    pub fn test(t: T) -> Context<TestContextData<T>> {
        Context::new(String::from(""), TestContextData::new(t))
    }
}
impl UnprotectedContext {
    pub fn test<T: Send + 'static>(t: T) -> UnprotectedContext {
        UnprotectedContext::from(Context::test(t))
    }
}
