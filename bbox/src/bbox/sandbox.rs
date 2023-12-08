use std::collections::HashMap;
use std::any::Any;

use crate::bbox::BBox;
use crate::policy::{Policy, AnyPolicy, NoPolicy}; //, Conjunction};
use crate::bbox::magic_box_fold;

// BBox and containers of it are MagicUnboxable.
pub enum MagicUnboxEnum {
    BBox(BBox<Box<dyn Any>, AnyPolicy>),
    Value(Box<dyn Any>),
    Vec(Vec<MagicUnboxEnum>),
    Struct(HashMap<String, MagicUnboxEnum>),
}
// Public: client code should Derive this, and can call magic_box_fold, but should not be able to call from_enum
pub trait MagicUnbox {
    type Out; //Lite form of struct
    fn to_enum(self) -> MagicUnboxEnum;
    fn from_enum(e: MagicUnboxEnum) -> Result<Self::Out, ()>;
}

fn compose_option_policies(policies_vec: Vec<Option<AnyPolicy>>) -> Result<Option<AnyPolicy>, ()>  {
    let filtered_policies: Vec<AnyPolicy> = policies_vec.clone()
                            .into_iter()
                            .filter(|option| option.is_some())
                            .map(|some| some.unwrap())
                            .collect();
    if filtered_policies.len() > 0 {
        let base = filtered_policies[0].clone(); 
        let composed_policy = filtered_policies
                            .into_iter()
                            .fold(base, |acc, elem|
                                acc.join(elem).unwrap());
        Ok(Some(composed_policy))
    } else if policies_vec.len() > 0 {
        Ok(None)
    } else {
        Err(())
    }
}

impl MagicUnboxEnum {
    pub fn enum_policy(&self) -> Result<Option<AnyPolicy>, ()> { 
        match self {
            MagicUnboxEnum::Value(_) => Ok(None),

            MagicUnboxEnum::BBox(bbox) => Ok(Some(bbox.policy().clone())), 

            MagicUnboxEnum::Vec(vec)  => {
                let policies_vec = vec
                                .into_iter()
                                .map(|e| e
                                    .enum_policy().unwrap()) // can't use '?' bc of interaction with Option and Result
                                .collect();
                compose_option_policies(policies_vec)
            }
            MagicUnboxEnum::Struct(hashmap) => {
                //iterate over hashmap, collect policies
                let policies_vec = hashmap
                                .into_iter()
                                .map(|(_, val)| val
                                    .enum_policy().unwrap())
                                .collect();
                compose_option_policies(policies_vec)
            }
        }
    }
}

impl MagicUnbox for i32 {
    type Out = i32; 
    fn to_enum(self) -> MagicUnboxEnum {
        MagicUnboxEnum::Value(Box::new(self))
    }
    fn from_enum(e: MagicUnboxEnum) -> Result<Self::Out, ()> {
        match e {
            MagicUnboxEnum::Value(v) => match v.downcast() {
                Err(_) => Err(()),
                Ok(v) => Ok(*v),
            },
            _ => Err(()),
        }
    }
}

impl MagicUnbox for u64 {
    type Out = u64; 
    fn to_enum(self) -> MagicUnboxEnum {
        MagicUnboxEnum::Value(Box::new(self))
    }
    fn from_enum(e: MagicUnboxEnum) -> Result<Self::Out, ()> {
        match e {
            MagicUnboxEnum::Value(v) => match v.downcast() {
                Err(_) => Err(()),
                Ok(v) => Ok(*v),
            },
            _ => Err(()),
        }
    }
}

impl MagicUnbox for f64 {
    type Out = f64; 
    fn to_enum(self) -> MagicUnboxEnum {
        MagicUnboxEnum::Value(Box::new(self))
    }
    fn from_enum(e: MagicUnboxEnum) -> Result<Self::Out, ()> {
        match e {
            MagicUnboxEnum::Value(v) => match v.downcast() {
                Err(_) => Err(()),
                Ok(v) => Ok(*v),
            },
            _ => Err(()),
        }
    }
}

impl MagicUnbox for String {
    type Out = String; 
    fn to_enum(self) -> MagicUnboxEnum {
        MagicUnboxEnum::Value(Box::new(self))
    }
    fn from_enum(e: MagicUnboxEnum) -> Result<Self::Out, ()> {
        match e {
            MagicUnboxEnum::Value(v) => match v.downcast() {
                Err(_) => Err(()),
                Ok(v) => Ok(*v),
            },
            _ => Err(()),
        }
    }
}

impl<T: 'static, P: Policy + Clone + 'static> MagicUnbox for BBox<T, P> {
    type Out = T;
    fn to_enum(self) -> MagicUnboxEnum { 
        MagicUnboxEnum::BBox(self.to_any_type_and_policy()) 
    }
    fn from_enum(e: MagicUnboxEnum) -> Result<Self::Out, ()> {
        match e {
            MagicUnboxEnum::Value(v) => match v.downcast() {
                Err(_) => Err(()),
                Ok(v) => Ok(*v),
            },
            _ => Err(()),
        }
    }
}

impl<S: MagicUnbox> MagicUnbox for Vec<S> {
    type Out = Vec<S::Out>;
    fn to_enum(self) -> MagicUnboxEnum {
         MagicUnboxEnum::Vec(self.into_iter().map(|s| s.to_enum()).collect())
    }
    fn from_enum(e: MagicUnboxEnum) -> Result<Self::Out, ()> {
        match e {
            MagicUnboxEnum::Vec(v) => {
                let mut result_vec = Vec::new();
                for element in v.into_iter() {
                    result_vec.push(S::from_enum(element)?);
                }
                Ok(result_vec)
            }
            _ => Err(()),
        }
    }
}

pub fn sandbox_execute<S: MagicUnbox, R, F: FnOnce(S::Out) -> R>(
    s: S,
    lambda: F,
) -> BBox<R, NoPolicy>{ //TODO should this return a Result? 

    let outer_boxed = magic_box_fold(s).unwrap(); 
    // cached_policy isn't necessarily the correct Policy for the whole struct - the outer_boxed policy inside AnyPolicy is the correct composed one
    let _cached_policy = outer_boxed.policy().clone(); 
    BBox::new(lambda(outer_boxed.into_temporary_unbox()), NoPolicy::new())
}

pub fn sandbox_combine<S1: MagicUnbox, S2: MagicUnbox, R, F: FnOnce(S1::Out, S2::Out) -> R>(
    s1: S1,
    s2: S2,
    lambda: F,
) -> BBox<R, NoPolicy> {
    let outer_boxed1 = magic_box_fold(s1).unwrap(); 
    let cached_policy1 = outer_boxed1.policy().clone();
    let outer_boxed2 = magic_box_fold(s2).unwrap(); 
    let cached_policy2 = outer_boxed2.policy().clone();
    let _composed_policy = cached_policy1.join(cached_policy2).unwrap(); 
    BBox::new(lambda(outer_boxed1.into_temporary_unbox(), 
                      outer_boxed2.into_temporary_unbox()),
              NoPolicy::new())
}

/*  
impl<'a, S> Sandboxable for &'a Vec<S>
where
    &'a S: Sandboxable,
{
    type Out = Vec<<&'a S as Sandboxable>::Out>;
    fn unbox(self) -> Self::Out {
        self.iter().map(|s| s.unbox()).collect()
    }
}

// TODO(artem): merge policies together or use lambda to acquire policy.
// Sandbox execute with a container of bboxes.
pub fn sandbox_execute<S: Sandboxable, R, F: FnOnce(S::Out) -> R>(
    s: S,
    lambda: F,
) -> BBox<R, NoPolicy> {

    BBox::new(lambda(s.unbox()), NoPolicy {})
}
// Need to generalize this to many arguments.
pub fn sandbox_combine<S1: Sandboxable, S2: Sandboxable, R, F: FnOnce(S1::Out, S2::Out) -> R>(
    s1: S1,
    s2: S2,
    lambda: F,
) -> BBox<R, NoPolicy> {
    BBox::new(lambda(s1.unbox(), s2.unbox()), NoPolicy {})
}
*/