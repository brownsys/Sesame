use std::any::{Any, TypeId};
use crate::context::UnprotectedContext;
use crate::policy::{AnyPolicyable, Direction, Joinable, Policy, Reason};

// Simplified policy interface that application code can implement.
// Application code should implement this trait unless they have reasons to implement Joinable manually.
// or if their policy is not Any (e.g. has non-static refs).
pub trait SimplePolicy: Send + Sync + Any {
    fn simple_name(&self) -> String;
    fn simple_check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool;
    fn simple_join_direct(&mut self, other: &mut Self) -> bool;
}

// Every simple policy is automatically Joinable.
impl<P: SimplePolicy + Sized> Joinable for P {
    fn direction_to<P2: AnyPolicyable>(&self, _p: &P2) -> Direction {
        if TypeId::of::<P>() == TypeId::of::<P2>() {
            Direction::Equal
        } else {
            Direction::Unrelated
        }
    }
    fn join_in<P2: AnyPolicyable>(&mut self, p: &mut P2, direction: Direction) -> bool {
        match direction {
            Direction::Equal => {
                let p: &mut dyn Any = p;
                self.join_direct(p.downcast_mut().unwrap())
            },
            _ => unreachable!("bad direction"),
        }
    }
    fn join_direct(&mut self, p: &mut Self) -> bool {
        self.simple_join_direct(p)
    }
}

// Every SimplePolicy is automatically a Policy.
impl<P: SimplePolicy> Policy for P {
    fn name(&self) -> String {
        self.simple_name()
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
        self.simple_check(context, reason)
    }
}