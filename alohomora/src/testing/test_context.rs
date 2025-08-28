use crate::bbox::BBox;
use crate::context::{Context, UnprotectedContext};
use crate::policy::{AnyPolicyDyn, NoPolicy};
use crate::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};
use crate::sesame_type::r#type::SesameTypeOut;
use crate::{SesameType, SesameTypeEnum};
use std::any::Any;

#[derive(Clone)]
pub struct TestContextData<T: Send + Any>(BBox<T, NoPolicy>);

impl<T: Send + Any> TestContextData<T> {
    pub fn new(t: T) -> Self {
        Self(BBox::new(t, NoPolicy {}))
    }
}

#[doc = "Library implementation of SesameTypeOut. Do not copy this docstring!"]
impl<T: Send + Any> SesameTypeOut for TestContextData<T> {
    type Out = T;
}

#[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
impl<T: Send + Any> SesameType for TestContextData<T> {
    fn to_enum(self) -> SesameTypeEnum {
        SesameTypeEnum::BBox(self.0.into_any_no_clone())
    }
    fn from_enum(e: SesameTypeEnum) -> Result<Self, ()> {
        if let SesameTypeEnum::Value(t) = e {
            return match t.downcast() {
                Ok(t) => Ok(*t),
                Err(_) => Err(()),
            };
        }
        Err(())
    }
    fn out_from_enum(e: SesameTypeEnum<dyn Any, dyn AnyPolicyDyn>) -> Result<Self::Out, ()> {
        if let SesameTypeEnum::Value(t) = e {
            return match t.downcast() {
                Ok(t) => Ok(*t),
                Err(_) => Err(()),
            };
        }
        Err(())
    }
}

#[rocket::async_trait]
impl<'a, 'r, T: Send + Any> FromBBoxRequest<'a, 'r> for TestContextData<T> {
    type BBoxError = ();
    async fn from_bbox_request(
        _request: BBoxRequest<'a, 'r>,
    ) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        todo!("TestContextData should not be actually constructed FromBBoxRequest because it is only used for testing")
    }
}

impl<T: Send + Any> Context<TestContextData<T>> {
    pub fn test(t: T) -> Context<TestContextData<T>> {
        Context::new(String::from(""), TestContextData::new(t))
    }
}
impl UnprotectedContext {
    pub fn test<T: Send + Any>(t: T) -> UnprotectedContext {
        UnprotectedContext::from(Context::test(t))
    }
}
