use crate::bbox::BBox;
use crate::policy::{AnyPolicy, AnyPolicyDyn, PolicyDyn};
use crate::sesame_type::dyns::{SesameDyn, SesameDynRelation};
use crate::sesame_type::helpers::compose_policies;
use std::any::Any;
use std::collections::HashMap;

// This provides a generic representation for values, bboxes, vectors, and structs mixing them.
pub enum SesameTypeEnum<T: SesameDyn + ?Sized = dyn Any, P: PolicyDyn + ?Sized = dyn AnyPolicyDyn>
{
    BBox(BBox<Box<T>, AnyPolicy<P>>),
    Value(Box<T>),
    Vec(Vec<SesameTypeEnum<T, P>>),
    Struct(HashMap<String, SesameTypeEnum<T, P>>),
}

impl<T: SesameDyn + ?Sized, P: PolicyDyn + ?Sized> SesameTypeEnum<T, P> {
    pub fn remove_bboxes2(self) -> (Self, Result<Option<AnyPolicy<P>>, ()>) {
        match self {
            SesameTypeEnum::Value(val) => (SesameTypeEnum::Value(val), Ok(None)),
            SesameTypeEnum::BBox(bbox) => {
                let (t, p) = bbox.consume();
                (SesameTypeEnum::Value(t), Ok(Some(p)))
            }
            SesameTypeEnum::Vec(vec) => {
                let mut ts = Vec::with_capacity(vec.len());
                let mut ps = Ok(None);
                for e in vec {
                    let (t, p) = e.remove_bboxes2();
                    ts.push(t);
                    ps = compose_policies(ps, p);
                }
                (SesameTypeEnum::Vec(ts), ps)
            }
            SesameTypeEnum::Struct(hashmap) => {
                let mut ts = HashMap::with_capacity(hashmap.len());
                let mut ps = Ok(None);
                for (k, e) in hashmap {
                    let (t, p) = e.remove_bboxes2();
                    ts.insert(k, t);
                    ps = compose_policies(ps, p);
                }
                (SesameTypeEnum::Struct(ts), ps)
            }
        }
    }

    // Coerces self into the given type provided it is a Value(...) of that type.
    pub fn coerce<R: Any>(self) -> Result<R, ()>
    where
        T: SesameDynRelation<R>,
    {
        match self {
            SesameTypeEnum::Value(v) => match v.upcast_box().downcast() {
                Ok(t) => Ok(*t),
                Err(_) => Err(()),
            },
            _ => Err(()),
        }
    }
}
