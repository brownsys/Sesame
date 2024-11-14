#[macro_export]
macro_rules! generate_context {
    ($context_name: tt, $user: tt) => {
        // #[derive(AlohomoraType)]
        pub struct BaseContext<U> {
            current_user: U,
            // TODO: DBConn for queries
        }

        type $context_name = BaseContext<$user>;

        // #[::rocket::async_trait]
        // impl<'a, 'r, U: FromBBoxReq> FromBBoxRequest<'a, 'r> for BaseContext<U> {
        //     type BBoxError = ();
        //     async fn from_bbox_request(request:crate::rocket::BBoxRequest<'a,'r> ,) -> crate::rocket::BBoxRequestOutcome<Self, Self::BBoxError> {
        //         let c = BaseContext {
        //             current_user: U::from_bbox_request(request),
        //         };
        //         request.route().and_then(|_|{
        //             Some(c)
        //         }).into_outcome((rocket::http::Status::InternalServerError, ()))
        //     }
        // }
    };
}

// Both of these are difficult because
// 1. we want to auto impl FrontendPolicy, BackendPolicy based on whether the user can come from frontend or backend
//     - but top level name must have it impled only if the user is impled -> must have a generic param
//     - we must impl the policy trait for the final name (only we know pred) -> must be defined in this crate too

#[macro_export]
/// Takes a list of role predicates and their reason checks & converts that into a simple RBAC policy 
/// named `name`.
macro_rules! access_control_policy {
    ($name: tt, $context_name: tt, $user: tt, $([$pred_fn: tt, $reason_check: expr]),+ $([$default_reason_check: expr])?) => {
        #[derive(Clone, Debug)]
        /// Auto-Generated Access Control Policy
        pub struct $name(AccessControlPolicy<$user>);
            
        impl alohomora::policy::Policy for $name {
            fn name(&self) -> String {
                // TODO: add more here
                format!("{} (auto-generated access control policy)", stringify!($name))
            }

            fn check(&self, context: &alohomora::context::UnprotectedContext, reason: alohomora::policy::Reason<'_>) -> bool {
                // TODO: downcast to correct context for accessing
                let context = context.downcast_ref::<$context_name>().unwrap();
                // TODO: should be other way around
                $(if self.0.owner.$pred_fn(context) {
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

        // impl<U: alohomora::policy::FromFrontend> alohomora::policy::FrontendPolicy for Base<U> 
        //     where Base<U>: alohomora::policy::Policy {
        //         fn from_cookie<'a, 'r>(
        //                 _: &str,
        //                 _: &'a rocket::http::Cookie<'static>,
        //                 request: &'a rocket::Request<'r>) -> Self where Self: Sized {
        //                 Self::from_request(request)
        //         }
        //         fn from_request<'a, 'r>(request: &'a rocket::Request<'r>) -> Self
        //                 where
        //                     Self: Sized {
        //             Base {
        //                 owner: U::from_request(request),
        //             }
        //         }
        // }

        // impl<U: alohomora::policy::FromSchema> alohomora::policy::SchemaPolicy for Base<U> 
        //     where Base<U>: alohomora::policy::Policy {
        //     fn from_row(table_name: &str, row: &Vec<mysql::Value>) -> Self
        //         where
        //             Self: Sized {
        //         Base { owner: U::from_row(table_name, row) }
        //     }
        // }
    }
}