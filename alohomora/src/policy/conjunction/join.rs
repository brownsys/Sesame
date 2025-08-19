use crate::policy::{AnyPolicyBB, AnyPolicyDyn, AnyPolicyable, Policy, PolicyAnd, PolicyDyn, PolicyDynRelation};

// Helper functions.
pub fn join_helper<P1: AnyPolicyable, P2: AnyPolicyable>(p1: &mut P1, p2: &mut P2) -> bool {
    //p1.join(p2.policy_type_enum().normalize())
    todo!()
}

pub fn join_dyn<P1: AnyPolicyable, P2: AnyPolicyable, PDyn: PolicyDyn + ?Sized>(mut p1: P1, mut p2: P2) -> AnyPolicyDyn<PDyn>
where PDyn: PolicyDynRelation<P1>,
      PDyn: PolicyDynRelation<P2> {
    // Try to join first direction.
    if join_helper(&mut p1, &mut p2) {
        return AnyPolicyDyn::new(p1);
    }

    // Try to join second direction.
    if join_helper(&mut p2, &mut p1) {
        return AnyPolicyDyn::new(p2);
    }

    // Stack.
    PDyn::and_policy(PolicyAnd::new(AnyPolicyDyn::new(p1), AnyPolicyDyn::new(p2)))
}

// Helper used only inside Sesame to deal with join certain type erased policies during fold.
pub fn join_dyn_any<PDyn: PolicyDyn + ?Sized>(mut p1: AnyPolicyDyn<PDyn>, mut p2: AnyPolicyDyn<PDyn>) -> AnyPolicyDyn<PDyn> {
    // Try to join first direction.
    if join_helper(&mut p1, &mut p2) {
        return p1;
    }

    // Try to join second direction.
    if join_helper(&mut p2, &mut p1) {
        return p2;
    }

    // Stack.
    PDyn::and_policy(PolicyAnd::new(p1, p2))
}


// TODO(babman): unify all joins in one API!
pub fn join<P1: AnyPolicyable, P2: AnyPolicyable>(p1: P1, p2: P2) -> AnyPolicyBB {
    join_dyn(p1, p2)
}