use rocket::data::Outcome;
use rocket::request::FromRequest;
use crate::{AlohomoraType, AlohomoraTypeEnum};
use crate::context::{Context, UnprotectedContext};
use crate::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};

#[derive(Clone, Debug)]
pub struct TestContextData<T: AlohomoraType>(T);

impl<T: AlohomoraType> TestContextData<T> {
    pub fn new(t: T) -> Self {
        Self(t)
    }
}

#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<T: AlohomoraType> AlohomoraType for TestContextData<T> {
    type Out = T::Out;

    fn to_enum(self) -> AlohomoraTypeEnum {
        self.0.to_enum()
    }

    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        T::from_enum(e)
    }
}

#[rocket::async_trait]
impl<'a, 'r, T: AlohomoraType + FromBBoxRequest<'a, 'r>> FromBBoxRequest<'a, 'r> for TestContextData<T> {
    type BBoxError = T::BBoxError;
    async fn from_bbox_request(request: BBoxRequest<'a, 'r>) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        match T::from_bbox_request(request).await {
            BBoxRequestOutcome::Success(t) => BBoxRequestOutcome::Success(TestContextData(t)),
            BBoxRequestOutcome::Failure(e) => BBoxRequestOutcome::Failure(e),
            BBoxRequestOutcome::Forward(f) => BBoxRequestOutcome::Forward(f),
        }
    }
}

impl<T: AlohomoraType> Context<TestContextData<T>>
where
    T: for<'a, 'r> FromBBoxRequest<'a, 'r>,
    T: Send + 'static,
{
    pub fn test(t: T) -> Context<TestContextData<T>> {
        Context::new(String::from(""), TestContextData::new(t))
    }
}

// #[cfg(test)]
impl UnprotectedContext {
    // For internal tests.
    pub fn empty() -> Self {
        UnprotectedContext {
            route: String::from(""),
            data: Box::new(Option::<()>::None),
        }
    }
    pub fn test<T: 'static>(data: T) -> Self {
        UnprotectedContext {
            route: String::from(""),
            data: Box::new(data),
        }
    }
}