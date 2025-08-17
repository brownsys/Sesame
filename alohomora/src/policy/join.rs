use crate::policy::{AnyPolicyBB, AnyPolicyDyn, AnyPolicyable, PolicyAnd, PolicyDyn, PolicyDynRelation};

// Direction to join along which.
pub enum Direction {
    Unrelated,
    Equal,
    Inner(Box<Direction>),
    Left(Box<Direction>),
    Right(Box<Direction>),
}

// Represents policies that can be joined with others.
pub trait Joinable {
    fn direction_to<P: AnyPolicyable>(&self, p: &P) -> Direction where Self: Sized;
    fn join_in<P: AnyPolicyable>(&mut self, p: &mut P, direction: Direction) -> bool where Self: Sized;
    fn join_direct(&mut self, p: &mut Self) -> bool where Self: Sized;
}

// Main entry point for policy conjunction.
pub fn join_dyn<P1: AnyPolicyable, P2: AnyPolicyable, PDyn: PolicyDyn + ?Sized>(mut p1: P1, mut p2: P2) -> AnyPolicyDyn<PDyn>
where PDyn: PolicyDynRelation<P1>,
      PDyn: PolicyDynRelation<P2> {
      //PDyn: PolicyDynRelation<PolicyAnd<P1, P2>>  {
    let direction = p1.direction_to(&p2);
    if let Direction::Unrelated = direction {
        let direction = p2.direction_to(&p1);
        if let Direction::Unrelated = direction {
            let p1 = AnyPolicyDyn::new(p1);
            let p2 = AnyPolicyDyn::new(p2);
            PDyn::and_policy(PolicyAnd::new(p1, p2))
        } else if p2.join_in(&mut p1, direction) {
            AnyPolicyDyn::new(p2)
        } else {
            panic!("Policy Join Failed");
        }
    } else {
        if p1.join_in(&mut p2, direction) {
            AnyPolicyDyn::new(p1)
        } else {
            panic!("Policy Join Failed");
        }
    }
}
pub fn join_dyn_any<PDyn: PolicyDyn + ?Sized>(mut p1: AnyPolicyDyn<PDyn>, mut p2: AnyPolicyDyn<PDyn>) -> AnyPolicyDyn<PDyn> {
    let direction = p1.direction_to(&p2);
    if let Direction::Unrelated = direction {
        let direction = p2.direction_to(&p1);
        if let Direction::Unrelated = direction {
            PDyn::and_policy(PolicyAnd::new(p1, p2))
        } else if p2.join_in(&mut p1, direction) {
            p2
        } else {
            panic!("Policy Join Failed");
        }
    } else {
        if p1.join_in(&mut p2, direction) {
            p1
        } else {
            panic!("Policy Join Failed");
        }
    }
}

pub fn join<P1: AnyPolicyable, P2: AnyPolicyable>(p1: P1, p2: P2) -> AnyPolicyBB {
    join_dyn(p1, p2)
}

// Implement this for your policies to indicate that they cannot be joined, only stacked.
#[macro_export]
macro_rules! Unjoinable {
    ($struct_name:ident$(<$($generics:tt),*> $(where $($clause:tt)*)?)?) => {
        impl$(<$($generics),*>)? $crate::policy::Joinable for $struct_name$(<$($generics),*>)? $($(where $($clause)*)?)? {
            fn direction_to<__P: $crate::policy::AnyPolicyable>(&self, p: &__P) -> $crate::policy::Direction
            where
                Self: Sized,
            {
                $crate::policy::Direction::Unrelated
            }

            fn join_in<__P: $crate::policy::AnyPolicyable>(&mut self, _p: &mut __P, _direction: $crate::policy::Direction) -> bool
            where
                Self: Sized,
            {
                unreachable!("join_in should not be called on unjoinable {}", stringify!($struct_name))
            }

            fn join_direct(&mut self, p: &mut Self) -> bool
            where
                Self: Sized,
            {
                unreachable!("join_direct should not be called on unjoinable {}", stringify!($struct_name))
            }
        }
    };
}

pub use Unjoinable;