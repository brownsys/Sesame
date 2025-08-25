use std::any::Any;

use crate::context::UnprotectedContext;
use crate::policy::NotAPolicyContainer;
use crate::policy::{
    Join, Reflective,
    UpgradableToAny,
};

// Enum describing why/where the policy check is invoked.
#[derive(Clone)]
pub enum Reason<'i> {
    DB(&'i str, Vec<&'i mysql::Value>), // The statement (with ?)
    TemplateRender(&'i str),            // Template name/path.
    Cookie(&'i str),                    // Cookie name.
    Redirect(&'i str),                  // Redirect path (before substitution).
    Response,                           // Returning a response.
    Custom(&'i dyn Any),                // Custom operation (via unbox(..)).
}

// Public facing Policy traits.
pub trait Policy: Send + Sync + Reflective + UpgradableToAny + Join {
    fn name(&self) -> String;
    // Policy check function!
    fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool;
}

// Simplified policy interface that application code can implement.
// Application code should implement this trait unless they have reasons to implement Joinable manually.
// or if their policy is not Any (e.g. has non-static refs).
pub trait SimplePolicy: Send + Sync + Any + NotAPolicyContainer {
    fn simple_name(&self) -> String;
    fn simple_check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool;
    fn simple_join_direct(&mut self, other: &mut Self);
}

// Every SimplePolicy is automatically a Policy that can be joined with instances of the same
// policy.
impl<P: SimplePolicy> Policy for P {
    fn name(&self) -> String {
        self.simple_name()
    }
    fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool {
        self.simple_check(context, reason)
    }
}

// Schema policies can be constructed from DB rows.
pub trait SchemaPolicy: Policy {
    fn from_row(table_name: &str, row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized;
}

// Front end policy can be constructed from HTTP requests and from cookies.
pub trait FrontendPolicy: Policy {
    fn from_request<'a, 'r>(request: &'a rocket::Request<'r>) -> Self
    where
        Self: Sized;

    fn from_cookie<'a, 'r>(
        name: &str,
        cookie: &'a rocket::http::Cookie<'static>,
        request: &'a rocket::Request<'r>,
    ) -> Self
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use crate::context::UnprotectedContext;
    use crate::policy::{AnyPolicy, JoinAPI, Policy, Reason, SimplePolicy};
    use std::collections::HashSet;

    #[derive(Clone)]
    pub struct BasicPolicy {
        owner: String,
    }
    impl BasicPolicy {
        pub fn new(owner: String) -> Self {
            Self { owner }
        }
    }
    impl SimplePolicy for BasicPolicy {
        fn simple_name(&self) -> String {
            format!("BasicPolicy(owner: {:?})", self.owner)
        }
        fn simple_check(&self, context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
            &self.owner == context.downcast_ref::<String>().unwrap()
        }
        fn simple_join_direct(&mut self, other: &mut Self) {
            if self.owner != other.owner {
                panic!("Bad owners");
            }
        }
    }

    #[derive(Clone, PartialEq)]
    pub struct ACLPolicy {
        owners: HashSet<String>,
    }
    impl SimplePolicy for ACLPolicy {
        fn simple_name(&self) -> String {
            format!("ACLPolicy(owners: {:?})", self.owners)
        }
        fn simple_check(&self, context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
            self.owners
                .contains(context.downcast_ref::<String>().unwrap())
        }
        fn simple_join_direct(&mut self, other: &mut Self) {
            self.owners = self
                .owners
                .intersection(&other.owners)
                .map(String::clone)
                .collect();
            if self.owners.len() == 0 {
                panic!("Unsat policy");
            }
        }
    }

    #[test]
    fn join_homog_policies() {
        //join policies of the same type
        let admin1 = String::from("Admin1");
        let admin2 = String::from("Admin2");
        let alice = String::from("Alice");
        let bob = String::from("Bob");

        let mult_acl: HashSet<String> =
            HashSet::from([alice.clone(), admin1.clone(), admin2.clone()]);
        let alice_acl: HashSet<String> = HashSet::from([alice.clone(), bob.clone()]);

        let acl_pol: ACLPolicy = ACLPolicy { owners: mult_acl };
        let alice_pol: ACLPolicy = ACLPolicy { owners: alice_acl };

        // combine in each direction
        let combined_pol = acl_pol.join(alice_pol);
        let specialized = combined_pol.specialize_top_ref::<ACLPolicy>().unwrap();

        // Users are allowed access to aggregated vector as expected
        let alice = UnprotectedContext::test(String::from("Alice"));
        assert!(combined_pol.check(&alice, Reason::Custom(&Box::new(()))));
        assert!(specialized.check(&alice, Reason::Custom(&Box::new(()))));

        // and correct users are disallowed access
        let admin1 = UnprotectedContext::test(String::from(&admin1));
        let admin2 = UnprotectedContext::test(String::from(&admin2));
        assert!(!combined_pol.check(&admin1, Reason::Custom(&Box::new(()))));
        assert!(!combined_pol.check(&admin2, Reason::Custom(&Box::new(()))));

        println!(
            "Final policy on aggregate of mixed policies: {}",
            combined_pol.name()
        );
    }

    #[test]
    #[should_panic]
    fn panic_policies() {
        //unsatisfiable policies of same type panic when combined
        let admin1 = String::from("Admin1");
        let admin2 = String::from("Admin2");
        let alice = String::from("Alice");
        let bob = String::from("Bob");

        let acl_pol = ACLPolicy {
            owners: HashSet::from([alice.clone(), admin1.clone(), admin2.clone()]),
        };
        let bob_pol = ACLPolicy {
            owners: HashSet::from([bob.clone()]),
        };

        // should panic - unsatisfiable policy
        let _combined_pol: AnyPolicy = acl_pol.join(bob_pol);
    }

    #[test]
    fn stack_policies() {
        let admin1 = String::from("Admin1");
        let admin2 = String::from("Admin2");
        let alice = String::from("Alice");

        let alice_acl = HashSet::from([alice.clone(), admin1.clone(), admin2.clone()]);

        let acl_pol = ACLPolicy { owners: alice_acl };
        let basic_pol = BasicPolicy::new(alice);

        // combine in each direction
        let combined_pol1: AnyPolicy = acl_pol.clone().join(basic_pol.clone());
        let combined_pol2: AnyPolicy = basic_pol.clone().join(acl_pol.clone());

        // Users are allowed access to aggregated vector as expected
        let alice = UnprotectedContext::test(String::from("Alice"));
        assert!(combined_pol1.check(&alice, Reason::Custom(&Box::new(()))));
        assert!(combined_pol2.check(&alice, Reason::Custom(&Box::new(()))));

        // and correct users are disallowed access
        let admin1 = UnprotectedContext::test(String::from(&admin1));
        let admin2 = UnprotectedContext::test(String::from(&admin2));

        assert!(!combined_pol1.check(&admin1, Reason::Custom(&Box::new(()))));
        assert!(!combined_pol2.check(&admin1, Reason::Custom(&Box::new(()))));

        assert!(!combined_pol1.check(&admin2, Reason::Custom(&Box::new(()))));
        assert!(!combined_pol2.check(&admin2, Reason::Custom(&Box::new(()))));

        println!("Final policy from mixed policies: {}", combined_pol1.name());
        println!("Final policy from mixed policies: {}", combined_pol2.name());
    }
}
