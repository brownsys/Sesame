use crate::rocket::BBoxRequest;
use std::{any::{Any, TypeId}, collections::HashSet};

use crate::context::Context;


// Public facing Policy traits.
pub trait Policy {
    fn name(&self) -> String;
    fn check(&self, context: &dyn Any) -> bool;
}
pub trait SchemaPolicy: Policy {
    fn from_row(row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized;
}
pub trait Conjunction<E>: Policy + Sized {
    fn join(&self, p2: &Self) -> Result<Self, E>;
}

pub trait FrontendPolicy: Policy + Send {
    fn from_request(request: &BBoxRequest<'_, '_>) -> Self
    where
        Self: Sized;
    // TODO(babman): from_cookie should become from_request.
    fn from_cookie() -> Self
    where
        Self: Sized;
}

// Any (owned) Policy.
trait TypeIdPolicyTrait: Policy + Any {
    fn clone(&self) -> Box<dyn TypeIdPolicyTrait>;
}
impl<P: Policy + Clone + 'static> TypeIdPolicyTrait for P {
    fn clone(&self) -> Box<dyn TypeIdPolicyTrait> {
        Box::new(self.clone())
    }
}

pub struct AnyPolicy {
    policy: Box<dyn TypeIdPolicyTrait>,
}
impl AnyPolicy {
    pub fn new<P: Policy + Clone + 'static>(p: P) -> Self {
        Self {
            policy: Box::new(p),
        }
    }
    pub fn is<P: Policy + 'static>(&self) -> bool {
        TypeId::of::<P>() == self.policy.as_ref().type_id()
    }
    pub fn specialize<P: Policy + 'static>(self) -> Result<P, String> {
        if self.is::<P>() {
            let raw = Box::into_raw(self.policy);
            let raw = raw as *mut P;
            Ok(*unsafe { Box::from_raw(raw) })
        } else {
            Err(format!(
                "Cannot convert '{}' to '{:?}'",
                self.name(),
                TypeId::of::<P>()
            ))
        }
    }
}
impl Policy for AnyPolicy {
    fn name(&self) -> String {
        format!("AnyPolicy({})", self.policy.name())
    }
    fn check(&self, context: &dyn Any) -> bool {
        self.policy.check(context)
    }
}
impl Clone for AnyPolicy { 
    fn clone(&self) -> Self {
        Self {
            policy: self.policy.clone() 
        }
    }
}
impl Conjunction<()> for AnyPolicy {
    fn join(&self, p2: &Self) -> Result<Self, ()> {
        //TODO(corinn) this is the key to reconciling MagicUnbox and Conjunction 
        // need to be able to call policy.join(p2.policy) when doing magic_fold

        /*
        let self_policy_type = self.policy.as_ref().type_id();
        let p2_policy_type = p2.policy.as_ref().type_id();
        if self_policy_type == p2_policy_type { //
            Ok(AnyPolicy::new(self.policy.join(p2.policy)))
        } else { 
        */
        //funky clone() syntax to disambiguate which Clone
        Ok(AnyPolicy::new(PolicyAnd::new(Clone::clone(&self), Clone::clone(&p2)))) 
        //}
    }
}

// NoPolicy can be directly discarded.
#[derive(Clone)]
pub struct NoPolicy {}
impl NoPolicy {
    pub fn new () -> Self {
        Self {}
    }
}
impl Policy for NoPolicy {
    fn name(&self) -> String {
        String::from("NoPolicy")
    }
    fn check(&self, _context: &dyn Any) -> bool {
        true
    }
}
impl FrontendPolicy for NoPolicy {
    fn from_request<'a, 'r>(_request: &'a BBoxRequest<'a, 'r>) -> Self { 
        Self {}
    }
    fn from_cookie() -> Self {
        Self {}
    }
}
impl Conjunction<()> for NoPolicy {
    fn join(&self, _p2: &Self) -> Result<Self, ()> {  
        Ok(NoPolicy { })
    } 
}

#[derive(Clone)]
pub struct PolicyAnd {
    p1: AnyPolicy,
    p2: AnyPolicy,
}
impl PolicyAnd {
    pub fn new(p1: AnyPolicy, p2: AnyPolicy) -> Self {
        Self { p1, p2 }
    }
}
impl Policy for PolicyAnd {
    fn name(&self) -> String {
        format!("({} AND {})", self.p1.name(), self.p2.name())
    }
    fn check(&self, context: &dyn Any) -> bool {
        self.p1.check(context) && self.p2.check(context)
    }
}
impl Conjunction<()> for PolicyAnd { 
    fn join(&self, p2: &Self) -> Result<Self, ()> {
        //TODO(corinn) recursively check component policies to see if any can be matched and joined
        Ok(
            PolicyAnd::new(
            AnyPolicy::new(Clone::clone(self)), 
            AnyPolicy::new(Clone::clone(p2))))
    }
} 

/*
// This is the previous version of PolicyAnd which takes in parameterized types rather than AnyPolicy

#[derive(Clone)]
pub struct PolicyAnd<P1: Policy, P2: Policy> {
    p1: P1,
    p2: P2,
}
impl<P1: Policy, P2: Policy> PolicyAnd<P1, P2> {
    pub fn new(p1: P1, p2: P2) -> Self {
        Self { p1, p2 }
    }
}
impl<P1: Policy, P2: Policy> Policy for PolicyAnd<P1, P2> {
    fn name(&self) -> String {
        format!("({} AND {})", self.p1.name(), self.p2.name())
    }
    fn check(&self, context: &dyn Any) -> bool {
        self.p1.check(context) && self.p2.check(context)
    }
}
impl<P1: SchemaPolicy, P2: SchemaPolicy> SchemaPolicy for PolicyAnd<P1, P2> {
    fn from_row(row: &Vec<mysql::Value>) -> Self {
        Self {
            p1: P1::from_row(row),
            p2: P2::from_row(row),
        }
    }
}
impl<P1: FrontendPolicy, P2: FrontendPolicy> FrontendPolicy for PolicyAnd<P1, P2> {
    fn from_request<'a, 'r>(request: &'a BBoxRequest<'a, 'r>) -> Self {
        Self {
            p1: P1::from_request(request),
            p2: P2::from_request(request),
        }
    }
    fn from_cookie() -> Self {
        Self {
            p1: P1::from_cookie(),
            p2: P2::from_cookie(),
        }
    }
}
/* Are Conjunction and PolicyAnd too redundant?
    basically a way to wrap non-matching types in order to join()
    can we make Conjunction more general to allow for this?
*/
impl <P1: Policy, P2: Policy> Conjunction<()> for PolicyAnd<P1, P2> { 
    fn join(&self, p2: &Self) -> Result<Self, ()> {
        //TODO(corinn) recursively check component policies to see if any can be matched and joined
        Ok(PolicyAnd::new(self.clone(), p2.clone()))
    }
} */

#[derive(Clone)]
pub struct PolicyOr<P1: Policy, P2: Policy> {
    p1: P1,
    p2: P2,
}
impl<P1: Policy, P2: Policy> PolicyOr<P1, P2> {
    pub fn new(p1: P1, p2: P2) -> Self {
        Self { p1, p2 }
    }
}
impl<P1: Policy, P2: Policy> Policy for PolicyOr<P1, P2> {
    fn name(&self) -> String {
        format!("({} OR {})", self.p1.name(), self.p2.name())
    }
    fn check(&self, context: &dyn Any) -> bool {
        self.p1.check(context) || self.p2.check(context)
    }
}
impl<P1: SchemaPolicy, P2: SchemaPolicy> SchemaPolicy for PolicyOr<P1, P2> {
    fn from_row(row: &Vec<mysql::Value>) -> Self {
        Self {
            p1: P1::from_row(row),
            p2: P2::from_row(row),
        }
    }
}
impl<P1: FrontendPolicy, P2: FrontendPolicy> FrontendPolicy for PolicyOr<P1, P2> {
    fn from_request<'a, 'r>(request: &'a BBoxRequest<'a, 'r>) -> Self {
        Self {
            p1: P1::from_request(request),
            p2: P2::from_request(request),
        }
    }
    fn from_cookie() -> Self {
        Self {
            p1: P1::from_cookie(),
            p2: P2::from_cookie(),
        }
    }
}