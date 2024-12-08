use alohomora::policy::Policy;

#[derive(Clone)]
enum TransitionState {
    First,
    Second,
}

#[derive(Clone)]
pub struct TransitionPolicy<A: Policy + Clone + 'static, B: Policy + Clone + 'static> {
    a: A,
    b: B,
    status: TransitionState,
    check_status_change: fn(&alohomora::policy::Reason<'_>) -> Option<TransitionState>,
}

impl<A: Policy + Clone, B: Policy + Clone> TransitionPolicy<A, B> {
    fn new(a: A, b: B, 
        initial_status: TransitionState, 
        check_status_change: fn(&alohomora::policy::Reason<'_>) -> Option<TransitionState>) -> Self {
            TransitionPolicy { a, b, status: initial_status, check_status_change }
    }
}

impl<A: Policy + Clone, B: Policy + Clone> Policy for TransitionPolicy<A, B> {
    fn name(&self) -> String {
        match self.status {
            TransitionState::First => format!("Transition Policy (From *{}* -> {})", self.a.name(), self.b.name()),
            TransitionState::Second => format!("Transition Policy (From {} -> *{}*)", self.a.name(), self.b.name()),
        }
    }

    fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized {
        // similar to policy_join() but just for each item
        todo!()
    }

    fn check(&self, context: &alohomora::context::UnprotectedContext, reason: alohomora::policy::Reason<'_>) -> bool {
        let new_status = (self.check_status_change)(&reason);

        // do actual check with old status
        let check = match self.status {
            TransitionState::First => self.a.check(context, reason),
            TransitionState::Second => self.b.check(context, reason),
        };

        // update status if it has changed by entering this region,
        // match new_status {
        //     None => (),
        //     Some(new_status) => self.status = new_status,
        // }

        check
    }
        
    crate::default_policy_join!();
}