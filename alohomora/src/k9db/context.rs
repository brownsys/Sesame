use std::marker::PhantomData;
use crate::{AlohomoraType, AlohomoraTypeEnum};
use crate::bbox::BBox;
use crate::context::{Context};
use crate::policy::{AnyPolicy, NoPolicy};
use crate::rocket::{BBoxRequest, BBoxRequestOutcome, FromBBoxRequest};

// Applications must implement this.
// They must also implement FromBBoxRequest for their implementation of this.
pub trait K9dbContextDataTrait {
    fn user(&self) -> BBox<Option<String>, AnyPolicy>;
    fn purpose(&self) -> BBox<Option<String>, AnyPolicy>;
}
impl K9dbContextDataTrait for () {
    fn user(&self) -> BBox<Option<String>, AnyPolicy> {
        BBox::new(None, AnyPolicy::new(NoPolicy {}))
    }
    fn purpose(&self) -> BBox<Option<String>, AnyPolicy> {
        BBox::new(None, AnyPolicy::new(NoPolicy {}))
    }
}

#[derive(Clone)]
pub struct K9dbContextData<T: K9dbContextDataTrait> {
    pub user: BBox<Option<String>, AnyPolicy>,
    pub purpose: BBox<Option<String>, AnyPolicy>,
    pub _t: PhantomData<T>,
}

#[derive(Clone)]
pub struct K9dbContextDataOut {
    pub user: Option<String>,
    pub purpose: Option<String>,
}

impl<T: K9dbContextDataTrait> AlohomoraType for K9dbContextData<T> {
    type Out = K9dbContextDataOut;
    fn to_enum(self) -> AlohomoraTypeEnum {
        AlohomoraTypeEnum::Vec(vec![self.user.to_enum(), self.purpose.to_enum()])
    }
    fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
        match e {
            AlohomoraTypeEnum::Vec(mut v) => {
                let purpose = v.pop().unwrap();
                let purpose = BBox::<Option<String>, AnyPolicy>::from_enum(purpose)?;
                let user = v.pop().unwrap();
                let user = BBox::<Option<String>, AnyPolicy>::from_enum(user)?;
                Ok(K9dbContextDataOut { user, purpose })
            }
            _ => Err(()),
        }
    }
}

#[rocket::async_trait]
impl<'a, 'r, T: K9dbContextDataTrait + FromBBoxRequest<'a, 'r>> FromBBoxRequest<'a, 'r> for K9dbContextData<T> {
    type BBoxError = T::BBoxError;
    async fn from_bbox_request(request: BBoxRequest<'a, 'r>) -> BBoxRequestOutcome<Self, Self::BBoxError> {
        let t = T::from_bbox_request(request).await;
        match t {
            BBoxRequestOutcome::Failure(e) => BBoxRequestOutcome::Failure(e),
            BBoxRequestOutcome::Forward(f) => BBoxRequestOutcome::Forward(f),
            BBoxRequestOutcome::Success(t) => {
                let data = K9dbContextData {
                    user: t.user(),
                    purpose: t.purpose(),
                    _t: PhantomData,
                };
                BBoxRequestOutcome::Success(data)
            },
        }
    }
}

// This is what the context is from the perspective of the HTTP routes and application code.
pub type K9dbContext<T/*: K9dbContextDataTrait*/> = Context<K9dbContextData<T>>;

// This is the unprotected context in the policy check function.
pub type UnprotectedK9dbContextData =  K9dbContextDataOut;