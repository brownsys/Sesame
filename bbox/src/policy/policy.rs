use crate::rocket::BBoxRequest;
use std::any::Any;
use crate::policy::AnyPolicy;

// Public facing Policy traits.
pub trait Policy {
    fn name(&self) -> String;
    fn check(&self, context: &dyn Any) -> bool;
    // Stream line join, find way to make join combine inside AndPolicy instead of stacking!
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()>;
    fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized;
}

// Schema policies can be constructed from DB rows.
pub trait SchemaPolicy: Policy {
    fn from_row(row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized;
}

// Front end policy can be constructed from HTTP requests and from cookies.
pub trait FrontendPolicy: Policy + Send {
    fn from_request(request: &BBoxRequest<'_, '_>) -> Self
        where
            Self: Sized;
    // TODO(babman): from_cookie should become from_request.
    fn from_cookie() -> Self
        where
            Self: Sized;
}

mod tests {
    use crate::policy::{Policy, PolicyAnd, AnyPolicy}; 
    use std::any::Any;
    use std::collections::{HashSet};

    pub struct ContextData {
        pub user: String,
    }
    impl ContextData {
        pub fn get_user(&self) -> &String {
            &self.user
        }
    }

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
        fn check(&self, context: &dyn Any) -> bool {
            let context: &ContextData = context.downcast_ref().unwrap();
            let user: &String = context.get_user();
            self.owner == user.clone()
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
        fn check(&self, context: &dyn Any) -> bool {
            let context: &ContextData = context.downcast_ref().unwrap();
            let user: &String = context.get_user();
            self.owners.contains(user)
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

    # [test]
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
        assert!(combined_pol.check(&ContextData{user: String::from("Alice")}));
        assert!(specialized.check(&ContextData{user: String::from("Alice")}));

        //and correct users are disallowed access
        let admin1 = ContextData{ user: admin1.clone()}; 
        let admin2 = ContextData{ user: admin2.clone()}; 
        assert!(!combined_pol.check(&admin1));
        assert!(!combined_pol.check(&admin2));
        
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
        assert!(combined_pol1.check(&ContextData{user: String::from("Alice")}));
        assert!(combined_pol2.check(&ContextData{user: String::from("Alice")}));

        //and correct users are disallowed access
        let admin1 = ContextData{ user: admin1.clone()}; 
        let admin2 = ContextData{ user: admin2.clone()}; 

        assert!(!combined_pol1.check(&admin1));
        assert!(!combined_pol2.check(&admin1));

        assert!(!combined_pol1.check(&admin2));
        assert!(!combined_pol2.check(&admin2));
        
        println!("Final policy from mixed policies: {}", combined_pol1.name());
        println!("Final policy from mixed policies: {}", combined_pol2.name());
    }
}