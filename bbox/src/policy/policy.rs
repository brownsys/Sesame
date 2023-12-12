use crate::rocket::BBoxRequest;
use std::any::{Any, TypeId};


// Public facing Policy traits.
pub trait Policy {
    fn name(&self) -> String;
    fn check(&self, context: &dyn Any) -> bool;
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()>;
    fn join_logic(&self, other: Self) -> Result<Self, ()> where Self: Sized;
}
pub trait SchemaPolicy: Policy {
    fn from_row(row: &Vec<mysql::Value>) -> Self
    where
        Self: Sized;
}

pub trait FrontendPolicy: Policy + Send {
    fn from_request(request: &BBoxRequest<'_, '_>) -> Self
    where
        Self: Sized;
    // TODO(babman): from_cookie should become from_request.
    fn from_cookie() -> Self
    where
        Self: Sized;
}

// Any (owned) Policy.
trait TypeIdPolicyTrait: Policy + Any {
    fn clone(&self) -> Box<dyn TypeIdPolicyTrait>;
}
impl<P: Policy + Clone + 'static> TypeIdPolicyTrait for P {
    fn clone(&self) -> Box<dyn TypeIdPolicyTrait> {
        Box::new(self.clone())
    }
}

pub struct AnyPolicy {
    policy: Box<dyn TypeIdPolicyTrait>,
}
impl AnyPolicy {
    pub fn new<P: Policy + Clone + 'static>(p: P) -> Self {
        Self {
            policy: Box::new(p),
        }
    }
    pub fn is<P: Policy + 'static>(&self) -> bool {
        TypeId::of::<P>() == self.policy.as_ref().type_id()
    }
    pub fn specialize<P: Policy + 'static>(self) -> Result<P, String> {
        if self.is::<P>() {
            let raw = Box::into_raw(self.policy);
            let raw = raw as *mut P;
            Ok(*unsafe { Box::from_raw(raw) })
        } else {
            Err(format!(
                "Cannot convert '{}' to '{:?}'",
                self.name(),
                TypeId::of::<P>()
            ))
        }
    }
}
impl Policy for AnyPolicy {
    fn name(&self) -> String {
        format!("AnyPolicy({})", self.policy.name())
    }
    fn check(&self, context: &dyn Any) -> bool {
        self.policy.check(context)
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        self.policy.join(other)        
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        self.policy.join(other) 
    }
}
impl Clone for AnyPolicy { 
    fn clone(&self) -> Self {
        Self {
            policy: self.policy.clone() 
        }
    }
}

// NoPolicy can be directly discarded.
#[derive(Clone)]
pub struct NoPolicy {}
impl NoPolicy {
    pub fn new () -> Self {
        Self {}
    }
}
impl Policy for NoPolicy {
    fn name(&self) -> String {
        String::from("NoPolicy")
    }
    fn check(&self, _context: &dyn Any) -> bool {
        true
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        Ok(AnyPolicy::new(NoPolicy::new()))
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        Ok(NoPolicy {  })
    }
    
}
impl FrontendPolicy for NoPolicy {
    fn from_request<'a, 'r>(_request: &'a BBoxRequest<'a, 'r>) -> Self { 
        Self {}
    }
    fn from_cookie() -> Self {
        Self {}
    }
}
/* 
impl Conjunction<()> for NoPolicy {
    fn join(&self, _p2: &Self) -> Result<Self, ()> {  
        Ok(NoPolicy { })
    } 
}
*/
#[derive(Clone)]
pub struct PolicyAnd {
    p1: AnyPolicy,
    p2: AnyPolicy,
}
impl PolicyAnd {
    pub fn new(p1: AnyPolicy, p2: AnyPolicy) -> Self {
        Self { p1, p2 }
    }
}

impl Policy for PolicyAnd {
    fn name(&self) -> String {
        format!("({} AND {})", self.p1.name(), self.p2.name())
    }
    fn check(&self, context: &dyn Any) -> bool {
        self.p1.check(context) && self.p2.check(context)
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
        todo!()
    }
}

#[derive(Clone)]
pub struct PolicyOr<P1: Policy, P2: Policy> {
    p1: P1,
    p2: P2,
}
impl<P1: Policy, P2: Policy> PolicyOr<P1, P2> {
    pub fn new(p1: P1, p2: P2) -> Self {
        Self { p1, p2 }
    }
}
impl<P1: Policy, P2: Policy> Policy for PolicyOr<P1, P2> {
    fn name(&self) -> String {
        format!("({} OR {})", self.p1.name(), self.p2.name())
    }
    fn check(&self, context: &dyn Any) -> bool {
        self.p1.check(context) || self.p2.check(context)
    }
    fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
        todo!()
    }
    fn join_logic(&self, other: Self) -> Result<Self, ()> {
      todo!()
    }
}
impl<P1: SchemaPolicy, P2: SchemaPolicy> SchemaPolicy for PolicyOr<P1, P2> {
    fn from_row(row: &Vec<mysql::Value>) -> Self {
        Self {
            p1: P1::from_row(row),
            p2: P2::from_row(row),
        }
    }
}
impl<P1: FrontendPolicy, P2: FrontendPolicy> FrontendPolicy for PolicyOr<P1, P2> {
    fn from_request<'a, 'r>(request: &'a BBoxRequest<'a, 'r>) -> Self {
        Self {
            p1: P1::from_request(request),
            p2: P2::from_request(request),
        }
    }
    fn from_cookie() -> Self {
        Self {
            p1: P1::from_cookie(),
            p2: P2::from_cookie(),
        }
    }
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
            Self{owner: owner}
        }
    }
    impl Policy for BasicPolicy {
        fn check(&self, context: &dyn Any) -> bool {
            let context: &ContextData = context.downcast_ref().unwrap();
            let user: &String = context.get_user();
            self.owner == user.clone()
        }
        fn name(&self) -> String {
            format!("BasicPolicy(owner: {:?})", self.owner) 
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
    impl ACLPolicy {
        pub fn new(owners: HashSet<String>) -> Self {
            Self{owners: owners}
        }
    }
    impl Policy for ACLPolicy {
        fn check(&self, context: &dyn Any) -> bool {
            let context: &ContextData = context.downcast_ref().unwrap();
            let user: &String = context.get_user();
            self.owners.contains(user)
        }
        fn name(&self) -> String {
            format!("ACLPolicy(owners: {:?})", self.owners) 
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
                Ok(ACLPolicy{owners: owners})
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

        let acl_pol = ACLPolicy::new(mult_acl); 
        let alice_pol = ACLPolicy::new(alice_acl); 
        
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

        let acl_pol = ACLPolicy::new(HashSet::from([alice.clone(), admin1.clone(), admin2.clone()])); 
        let bob_pol = ACLPolicy::new(HashSet::from([bob.clone()])); 
        
        //should panic - unsatisfiable policy
        let _combined_pol: AnyPolicy = acl_pol.join(AnyPolicy::new(bob_pol.clone())).unwrap();
    }

    # [test]
    fn stack_policies(){
        let admin1 = String::from("Admin1");
        let admin2 = String::from("Admin2");
        let alice = String::from("Alice");
        
        let alice_acl = HashSet::from([alice.clone(), admin1.clone(), admin2.clone()]);

        let acl_pol = ACLPolicy::new(alice_acl); 
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