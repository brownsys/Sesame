use std::any::Any;
use std::boxed::Box;
use crate::context::UnprotectedContext;
use crate::policy::AnyPolicy;

pub trait CloneableAny : Any {
    fn any_clone(&self) -> Box<dyn CloneableAny>;
    fn cast(&self) -> &dyn Any;
}
impl<A: Clone + 'static> CloneableAny for A {
    fn any_clone(&self) -> Box<dyn CloneableAny> {
        Box::new(self.clone())
    }
    fn cast(&self) -> &dyn Any {
        self
    }
}

// Enum describing why/where the policy check is invoked.
pub enum Reason<'i> {
    DB(&'i str, Vec<mysql::Value>),  // The statement (with ?)
    TemplateRender(&'i str),             // Template name/path.
    Cookie(&'i str),                     // Cookie name.
    Redirect(&'i str),                   // Redirect path (before substitution).
    Response,                            // Returning a response.
    Custom(Box<dyn CloneableAny>),       // Custom operation (via unbox(..)).
}

impl<'i> Clone for Reason<'i> {
    fn clone(&self) -> Self {
        match self {
            Reason::DB(query, params) => Reason::DB(query, params.clone()),
            Reason::TemplateRender(template_name) => Reason::TemplateRender(template_name),
            Reason::Cookie(cookie_name) => Reason::Cookie(cookie_name),
            Reason::Redirect(path) => Reason::Redirect(path),
            Reason::Response => Reason::Response,
            Reason::Custom(b) => Reason::Custom((*b).any_clone()),
        }
    }
}

// Public facing Policy traits.
pub trait Policy : Send + Sync {
    fn name(&self) -> String;
    fn check(&self, context: &UnprotectedContext, reason: Reason<'_>) -> bool;
    // TODO(babman): Stream line join, find way to make join combine inside AndPolicy instead of stacking!
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()>;
    fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized;
    // for casting to any policy.
    fn into_any(self) -> AnyPolicy where Self: Sized {
        panic!("unreachable code");
    }
}

// Schema policies can be constructed from DB rows.
pub trait SchemaPolicy: Policy {
    fn from_row(table_name: &str, row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized;
}

// Front end policy can be constructed from HTTP requests and from cookies.
pub trait FrontendPolicy: Policy + Send {
    fn from_request<'a, 'r>(request: &'a rocket::Request<'r>) -> Self
        where
            Self: Sized;

    fn from_cookie<'a, 'r>(
        name: &str,
        cookie: &'a rocket::http::Cookie<'static>,
        request: &'a rocket::Request<'r>) -> Self where Self: Sized;
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use crate::context::UnprotectedContext;
    use crate::policy::{Policy, PolicyAnd, AnyPolicy, Reason};

    #[derive(Clone)]
    pub struct BasicPolicy {
        owner: String,
    }
    impl BasicPolicy {
        pub fn new(owner: String) -> Self {
            Self { owner }
        }
    }
    impl Policy for BasicPolicy {
        fn name(&self) -> String {
            format!("BasicPolicy(owner: {:?})", self.owner)
        }
        fn check(&self, context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
            &self.owner == context.downcast_ref::<String>().unwrap()
        }
        fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> { 
            if other.is::<BasicPolicy>() { //Policies are combinable
                let other = other.specialize::<BasicPolicy>().unwrap();
                Ok(AnyPolicy::new(self.join_logic(other)?)) 
            } else {                    //Policies must be stacked
                Ok(AnyPolicy::new(
                    PolicyAnd::new(
                        AnyPolicy::new(self.clone()),
                        other)))
            }
        }
        fn join_logic(&self, other: Self) -> Result<Self, ()> {
            if self.owner == other.owner {
                Ok(Self::new(self.owner.clone()))
            } else {
                Err(())
            }
        }
    }

    #[derive(Clone, PartialEq)]
    pub struct ACLPolicy {
        owners: HashSet<String>,
    }
    impl Policy for ACLPolicy {
        fn name(&self) -> String {
            format!("ACLPolicy(owners: {:?})", self.owners)
        }
        fn check(&self, context: &UnprotectedContext, _reason: Reason<'_>) -> bool {
            self.owners.contains(context.downcast_ref::<String>().unwrap())
        }
        fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> { 
            if other.is::<ACLPolicy>() { //Policies are combinable
                let other = other.specialize::<ACLPolicy>().unwrap();
                Ok(AnyPolicy::new(self.join_logic(other)?))
            } else {                    //Policies must be stacked
                Ok(AnyPolicy::new(
                    PolicyAnd::new(
                        AnyPolicy::new(self.clone()),
                        other)))
            }
        }
        fn join_logic(&self, other: Self) -> Result<Self, ()> {
            let intersection: HashSet<_> = self.owners.intersection(&other.owners).collect();
            let owners: HashSet<String> = intersection.into_iter().map(|owner| owner.clone()).collect(); 
            if owners.len() > 0 {
                Ok(ACLPolicy { owners })
            } else {
                Err(())
            }
        }
    }

    #[test]
    fn join_homog_policies(){ //join policies of the same type
        let admin1 = String::from("Admin1");
        let admin2 = String::from("Admin2");
        let alice = String::from("Alice");
        let bob = String::from("Bob");

        let mult_acl: HashSet<String> = HashSet::from([alice.clone(), admin1.clone(), admin2.clone()]);
        let alice_acl: HashSet<String> = HashSet::from([alice.clone(), bob.clone()]);

        let acl_pol = ACLPolicy { owners: mult_acl };
        let alice_pol = ACLPolicy { owners: alice_acl };
        
        //combine in each direction
        let combined_pol: AnyPolicy = acl_pol.join(AnyPolicy::new(alice_pol.clone())).unwrap();

        let specialized = combined_pol.clone().specialize::<ACLPolicy>().unwrap(); 
            
        // Users are allowed access to aggregated vector as expected
        let alice = UnprotectedContext::test(String::from("Alice"));
        assert!(combined_pol.check(&alice, Reason::Custom(Box::new(()))));
        assert!(specialized.check(&alice, Reason::Custom(Box::new(()))));

        // and correct users are disallowed access
        let admin1 = UnprotectedContext::test(String::from(&admin1));
        let admin2 = UnprotectedContext::test(String::from(&admin2));
        assert!(!combined_pol.check(&admin1, Reason::Custom(Box::new(()))));
        assert!(!combined_pol.check(&admin2, Reason::Custom(Box::new(()))));
        
        println!("Final policy on aggregate of mixed policies: {}", combined_pol.name());
    }

    #[test]
    #[should_panic]
    fn panic_policies(){ //unsatisfiable policies of same type panic when combined
        let admin1 = String::from("Admin1");
        let admin2 = String::from("Admin2");
        let alice = String::from("Alice");
        let bob = String::from("Bob");

        let acl_pol = ACLPolicy { owners: HashSet::from([alice.clone(), admin1.clone(), admin2.clone()]) };
        let bob_pol = ACLPolicy { owners: HashSet::from([bob.clone()]) };
        
        //should panic - unsatisfiable policy
        let _combined_pol: AnyPolicy = acl_pol.join(AnyPolicy::new(bob_pol.clone())).unwrap();
    }

    # [test]
    fn stack_policies(){
        let admin1 = String::from("Admin1");
        let admin2 = String::from("Admin2");
        let alice = String::from("Alice");
        
        let alice_acl = HashSet::from([alice.clone(), admin1.clone(), admin2.clone()]);

        let acl_pol = ACLPolicy { owners: alice_acl };
        let basic_pol = BasicPolicy::new(alice); 
        
        //combine in each direction
        let combined_pol1: AnyPolicy = acl_pol.join(AnyPolicy::new(basic_pol.clone())).unwrap();
        let combined_pol2: AnyPolicy = basic_pol.join(AnyPolicy::new(acl_pol)).unwrap();
            
        // Users are allowed access to aggregated vector as expected
        let alice = UnprotectedContext::test(String::from("Alice"));
        assert!(combined_pol1.check(&alice, Reason::Custom(Box::new(()))));
        assert!(combined_pol2.check(&alice, Reason::Custom(Box::new(()))));

        // and correct users are disallowed access
        let admin1 = UnprotectedContext::test(String::from(&admin1));
        let admin2 = UnprotectedContext::test(String::from(&admin2));

        assert!(!combined_pol1.check(&admin1, Reason::Custom(Box::new(()))));
        assert!(!combined_pol2.check(&admin1, Reason::Custom(Box::new(()))));

        assert!(!combined_pol1.check(&admin2, Reason::Custom(Box::new(()))));
        assert!(!combined_pol2.check(&admin2, Reason::Custom(Box::new(()))));
        
        println!("Final policy from mixed policies: {}", combined_pol1.name());
        println!("Final policy from mixed policies: {}", combined_pol2.name());
    }
}
