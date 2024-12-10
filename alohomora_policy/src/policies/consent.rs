#[macro_export]
/// Takes a list of role predicates and their reason checks & converts that into a simple RBAC policy 
/// named `name`.
macro_rules! consent_policy {
    ($name: tt,
     $user: tt,
     $([$consent_val: tt $(|| $next_consent_val: tt)*, $reason_check: expr]),* 
     $(($default_reason_check: expr))? $(; $user_combine_fn: expr)?) => {

        /// Auto-Generated Access Control Policy
        // TODO: remove Debug, i dont think its needed
        #[derive(Clone, Debug)]
        pub struct $name {
            pub owner: $user,
        }

        impl alohomora::policy::DefaultWithUser for $name {
            type User = $user;
            fn make(user: Self::User) -> Self {
                $name { owner: user }
            }
        }
            
        impl alohomora::policy::Policy for $name {
            fn name(&self) -> String {
                // TODO: add more here
                format!("{} (auto-generated consent policy)", stringify!($name))
            }

            fn check(&self, context: &alohomora::context::UnprotectedContext, reason: alohomora::policy::Reason<'_>) -> bool {
                // TODO: downcast to correct context for accessing
                // let context = context.downcast_ref::<<$context_name as alohomora::AlohomoraType>::Out>().unwrap();
                // TODO: should be other way around
                $(if self.owner.$consent_val $(|| self.owner.$next_consent_val)* {
                    // TODO: should it be okay if this check fails?
                    return $reason_check(reason);
                })*

                return $($default_reason_check(reason) ||)? false;
            }

            #[allow(unreachable_code)]
            fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized {
                $(return Ok(<Self as alohomora::policy::DefaultWithUser>::make($user_combine_fn(&self.owner, &other.owner)));)?
                return todo!();
            }
                
            $crate::default_policy_join!();
        }
    }
}