

#[macro_export]
macro_rules! access_control_policy {
    ($name: tt, $([$pred: tt, $reason_check: expr])+) => {
        #[derive(Clone)]
        /// Auto-Generated Access Control Policy
        pub struct $name {

        }

        // Auto-Generated Access Control Policy
        impl alohomora::policy::Policy for $name {
            fn name(&self) -> String {
                todo!()
            }

            fn check(&self, context: &alohomora::context::UnprotectedContext, reason: alohomora::policy::Reason<'_>) -> bool {
                $(if $pred {
                    // TODO: should it be okay if this check fails?
                    return $reason_check(reason);
                })*

                return false;
            }
            
            fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized {
                // similar to policy_join() but just for each item
                todo!()
            }
            $crate::default_policy_join!();
        }
    }
}