// use alohomora::policy::{FromFrontend, FrontendPolicy};


#[macro_export]
macro_rules! generate_context {
    ($context_name: tt, $user: tt) => {
        // #[derive(AlohomoraType)]

        pub type $context_name = hello::BaseContext<$user>;

        mod hello {
            use rocket::outcome::IntoOutcome;

            pub(crate) struct BaseContext<U> {
                current_user: U,
                // TODO: DBConn for queries
            }

            #[::rocket::async_trait]
            impl<'a, 'r, U: alohomora::policy::FromBBoxReq> alohomora::rocket::FromBBoxRequest<'a, 'r> for BaseContext<U> {
                type BBoxError = ();
                async fn from_bbox_request(request: alohomora::rocket::BBoxRequest<'a,'r> ,) -> alohomora::rocket::BBoxRequestOutcome<Self, Self::BBoxError> {
                    let c = BaseContext {
                        current_user: U::from_bbox_request(request),
                    };
                    request.route().and_then(|_|{
                        Some(c)
                    }).into_outcome((rocket::http::Status::InternalServerError, ()))
                }
            }
        }
    };
}

#[macro_export]
/// Takes a list of role predicates and their reason checks & converts that into a simple RBAC policy 
/// named `name`.
macro_rules! access_control_policy {
    ($name: tt, 
     $context_name: tt, 
     $user: tt,
     $([$pred_fn: tt $(|| $next_pred_fn: tt)*, $reason_check: expr]),* 
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
                format!("{} (auto-generated access control policy)", stringify!($name))
            }

            fn check(&self, context: &alohomora::context::UnprotectedContext, reason: alohomora::policy::Reason<'_>) -> bool {
                // TODO: downcast to correct context for accessing
                let context = context.downcast_ref::<<$context_name as alohomora::AlohomoraType>::Out>().unwrap();
                // TODO: should be other way around
                $(if self.owner.$pred_fn(context) $(|| self.owner.$next_pred_fn(context))* {
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