#[macro_export]
macro_rules! generate_context {
    ($context_name: tt, $user: tt) => {
        // #[derive(AlohomoraType)]
        pub struct $context_name {
            current_user: $user,
            // TODO: DBConn for queries
        }
    };
}

#[macro_export]
/// Takes a list of role predicates and their reason checks & converts that into a simple RBAC policy 
/// named `name`.
macro_rules! access_control_policy {
    ($name: tt, $context_name: tt, $user: tt, $([$pred_fn: tt, $reason_check: expr]),+ $([$default_reason_check: expr])?) => {
        #[derive(Clone, Debug)]
        /// Auto-Generated Access Control Policy
        pub struct Base<U> {
            // data corresponding to the data properties that will decide access (e.g. owner)
            owner: U,

        }

        type $name = Base<$user>;
            
        impl alohomora::policy::Policy for $name {
            fn name(&self) -> String {
                // TODO: add more here
                format!("{} (auto-generated access control policy)", stringify!($name))
            }

            fn check(&self, context: &alohomora::context::UnprotectedContext, reason: alohomora::policy::Reason<'_>) -> bool {
                // TODO: downcast to correct context for accessing
                let context = context.downcast_ref::<$context_name>().unwrap();
                // TODO: should be other way around
                $(if self.owner.$pred_fn(context) {
                    // TODO: should it be okay if this check fails?
                    return $reason_check(reason);
                })*

                return $($default_reason_check(reason) ||)? false;
            }
                
            fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized {
                // similar to policy_join() but just for each item
                todo!()
            }
                
            $crate::default_policy_join!();
        }

        impl<U: FromFrontend> FrontendPolicy for Base<U> 
            where Base<U>: Policy {
                fn from_cookie<'a, 'r>(
                        _: &str,
                        _: &'a rocket::http::Cookie<'static>,
                        request: &'a rocket::Request<'r>) -> Self where Self: Sized {
                        Self::from_request(request)
                }
                fn from_request<'a, 'r>(request: &'a rocket::Request<'r>) -> Self
                        where
                            Self: Sized {
                    Base {
                        owner: U::from_request(request),
                    }
                }
        }

        // TODO: backend & ORM policy
    }
}