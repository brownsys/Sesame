use crate::bbox::BBox;
use crate::policy::{Policy, AnyPolicy, NoPolicy}; //Conjunction};
use std::convert::TryFrom;

use crate::bbox::{MagicUnbox, MagicUnboxEnum};
//use crate::context::Context; 

// TODO(artem): think about how both of these interact with the policies
//              we likely need some sort of foldable trait for each direction
//              with a combine and a default function.

// Move BBox inside and outside a vec.
impl<T, P: Policy + Clone> From<BBox<Vec<T>, P>> for Vec<BBox<T, P>> {
    fn from(x: BBox<Vec<T>, P>) -> Vec<BBox<T, P>> {
        let p = x.p;
        x.t.into_iter().map(|t| BBox::new(t, p.clone())).collect()
    }
}
impl<T, P: Policy> TryFrom<Vec<BBox<T, P>>> for BBox<Vec<T>, P> {
    type Error = &'static str;
    fn try_from(mut value: Vec<BBox<T, P>>) -> Result<Self, Self::Error> {
        match value.pop() {
            None => Err("Folding out empty vector"),
            Some(v) => {
                let mut vec: Vec<T> = value.into_iter().map(|b| b.t).collect();
                vec.push(v.t);
                Ok(BBox::new(vec, v.p))
            }
        }
    }
}

/* ---------------------------------------------------------------- */

//intermediate but over-specialized box folding - no recursion for inner boxes
pub fn fold_out_box<T: Clone, P: Policy + Clone + 'static> // + Conjunction<()>>
                    (bbox_vec : Vec<BBox<T, P>>) -> Result<BBox<Vec<T>, AnyPolicy>, ()> {
    let values = bbox_vec
                        .clone().into_iter()
                        .map(|bbox| bbox.clone().temporary_unbox().clone())
                        .collect();
    let policies_vec: Vec<AnyPolicy> = bbox_vec
                        .clone().into_iter()
                        .map(|bbox| AnyPolicy::new(bbox.clone().policy().clone()))
                        .collect();
    if policies_vec.len() > 0 {
        let base = policies_vec[0].clone(); 
        let composed_policy = policies_vec
                            .into_iter()
                            .fold(base,  //base 0th instead of reduce bc don't need to unwrap()
                                |acc, elem|
                                acc.join(elem).unwrap());
        Ok(BBox::new(values, composed_policy))
    } else {
        //TODO(corinn)
        //Desired behavior: BBox around empty vec + empty constructor of Policy P
        //Ok(BBox::new(values, P::new())) 
        Err(())
    }
}


pub fn fold_in_box<T: Clone, P: Policy + Clone> //+ Conjunction<()>>
                    (boxed_vec : BBox<Vec<T>, P>) -> Vec<BBox<T, P>> {
    let policy = boxed_vec.clone().policy().clone(); 
    boxed_vec.clone().temporary_unbox().clone()
            .into_iter()
            .map(|item: T| BBox::new(item, policy.clone()))
            .collect()
}

/* ---------------------------------------------------------------- */

pub fn magic_box_fold<S: MagicUnbox>(strct: S) -> Result<BBox<S::Out, AnyPolicy>, ()> {
    let e = strct.to_enum(); 
    let composed_policy = e.enum_policy()?; //Err propagates if policy composition fails
    let e = magic_fold_helper(e); //remove bbox
    let e = S::from_enum(e)?; //convert back to defined S::Out type
    match composed_policy {
        Some(policy) => Ok(BBox::new(e, policy)), 
        None => Ok(BBox::new(e, AnyPolicy::new(NoPolicy::new())))
    }
}

pub(crate) fn magic_fold_helper(e: MagicUnboxEnum) -> MagicUnboxEnum {
    match e {
        MagicUnboxEnum::Value(val) => MagicUnboxEnum::Value(val), 
        MagicUnboxEnum::BBox(bbox) => MagicUnboxEnum::Value(bbox.t), //remove bbox        
        MagicUnboxEnum::Vec(vec) => {
            MagicUnboxEnum::Vec(vec.into_iter().map(|e| magic_fold_helper(e)).collect())
        }
        MagicUnboxEnum::Struct(hashmap) => MagicUnboxEnum::Struct(
            hashmap
                .into_iter()
                .map(|(key, val)| (key, magic_fold_helper(val)))
                .collect(),
        ),
    }
}

mod tests {
    use crate::policy::{Policy, PolicyAnd, AnyPolicy}; // , Conjunction};
    use crate::bbox::{BBox, magic_box_fold, fold_out_box, MagicUnbox, MagicUnboxEnum};
    use crate::context::Context;

    use std::any::Any;
    use std::collections::{HashSet, HashMap};

    pub struct ContextData {
        pub user: String,
    }
    impl ContextData {
        pub fn get_user(&self) -> &String {
            &self.user
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
    /*
    impl Conjunction<()> for ACLPolicy {
        fn join(&self, p2: &Self) -> Result<Self, ()> {     
            todo!()
        }
    }
    */
    #[derive(Clone, PartialEq, Debug)]
    pub struct BoxedStruct {
        pub score: BBox<u64, ACLPolicy>,
    }
    impl BoxedStruct {
        pub fn new(score: u64, policy: ACLPolicy) -> Self {
            Self {
                score: BBox::new(score, policy)
            }
        }
    }

    #[derive(PartialEq, Debug)]
    pub struct BoxedStructLite {
        score: u64,
    }
    impl BoxedStructLite {
        pub fn new(score: u64) -> Self {
            Self {
                score: score,
            }
        }
    }
    impl MagicUnbox for BoxedStruct {
        type Out = BoxedStructLite; 
        fn to_enum(self) -> MagicUnboxEnum {
            let hashmap = HashMap::from([
                (String::from("score"), self.score.to_enum()),
            ]);
            MagicUnboxEnum::Struct(hashmap)  
        }
        fn from_enum(e: MagicUnboxEnum) -> Result<Self::Out, ()> {
            match e {
                MagicUnboxEnum::Struct(mut hashmap) => Ok(Self::Out {
                    score: <u64 as MagicUnbox>::from_enum(hashmap.remove("score").unwrap())?,
                }),
                _ => Err(()),
            }
        }
    }
    
    #[test]
    fn fold_out_box_test(){ //test * manual * folding
        
        let alice = String::from("Alice");
        let bob = String::from("Bob"); 
        let allen = String::from("Allen");

        let pol1 = ACLPolicy::new(HashSet::from([alice.clone(), bob.clone()]));
        let pol2 = ACLPolicy::new(HashSet::from([alice.clone(), allen.clone()]));

        let mut orig_vec = Vec::new();
        let mut expected = Vec::new();
        let mut i = 0;

        while i < 20 {
            if i % 2 == 0 {
                orig_vec.push(BBox::new(i, pol1.clone()));
            } else {
                orig_vec.push(BBox::new(i, pol2.clone()));
            }
            expected.push(i);
            i = i + 1;
        }

        // Values are correct
        let folded = fold_out_box(orig_vec).unwrap();
        assert_eq!(expected, folded.t);

        //ACLPolicy conjunction worked
        let allowed = ContextData{ user: alice}; 
        let unallowed1 =  ContextData{ user: bob}; 
        let unallowed2 =  ContextData{ user: allen}; 

        assert!(folded.policy().check(&allowed));
        assert!(!folded.policy().check(&unallowed1));
        assert!(!folded.policy().check(&unallowed2));
    }

    #[test] 
    fn fold_raw_data() {
        let alice = String::from("Alice");
        let num = 28; 
        let deci = 32.0; 

        //Call magic_box_fold on unboxed data w/ MagicUnbox impl'd
        let alice_res = magic_box_fold(alice.clone());
        let num_res = magic_box_fold(num.clone());
        let deci_res = magic_box_fold(deci.clone());
        //The aggregated data is as expected
        assert_eq!(alice_res.as_ref().unwrap().t, alice.clone());
        assert_eq!(num_res.as_ref().unwrap().t, num.clone());
        assert_eq!(deci_res.as_ref().unwrap().t, deci.clone());

        // Any user is allowed access to aggregated vector bc resultant BBox has NoPolicy
        let allowed = ContextData{ user: String::from("")}; 
        //arbitrary user can access all results
        assert!(alice_res.as_ref().unwrap()
                    .policy().check(&allowed));
        assert!(num_res.as_ref().unwrap()
                    .policy().check(&allowed));
        assert!(deci_res.as_ref().unwrap()
                    .policy().check(&allowed));
    }

    #[test] 
    fn fold_nobox_vec() {
        let admin1 = String::from("Admin1");
        let alice = String::from("Alice");
        let bob = String::from("Bob"); 
        let allen = String::from("Allen");
        
        let mut orig_vec = Vec::new();
        orig_vec.extend([alice.clone(), bob.clone(), allen.clone()]);
        
        let mut res_vec = Vec::new(); 
        res_vec.extend([alice.clone(), bob.clone(), allen.clone()]);
            
        //Call to magic_box_fold 
        let agg = magic_box_fold(orig_vec);
    
        //The aggregated data is as expected
        assert_eq!(agg.as_ref().unwrap().t, res_vec);
    
        // Any user is allowed access to aggregated vector bc result is NoPolicy
        let allowed_admin1 = ContextData{ user: admin1.clone()}; 
        //anyone can access 
        assert!(agg.as_ref().unwrap()
                    .policy().check(&allowed_admin1));
    }

    #[test]
    #[should_panic]
    fn fold_unsatisfiable_boxes_vec() {
        let admin1 = String::from("Admin1");
        let admin2 = String::from("Admin2");
        let alice = String::from("Alice");
        let bob = String::from("Bob"); 
        let allen = String::from("Allen");
        
        let alice_acl = HashSet::from([alice.clone()]);
        let bob_acl = HashSet::from([bob.clone()]);
        let allen_acl = HashSet::from([allen.clone(), admin1.clone(), admin2.clone()]);
        
        let mut boxed_vec = Vec::new();
        boxed_vec.extend([BBox::new(100, ACLPolicy::new(alice_acl)), 
                                    BBox::new(99, ACLPolicy::new(bob_acl)), 
                                    BBox::new(95, ACLPolicy::new(allen_acl))]);
        
        let mut unboxed_vec = Vec::new(); 
        unboxed_vec.extend([100, 99, 95]);
            
        //Call to magic_box_fold -> will panic bc Policy join() was unsuccessful
        let _agg = magic_box_fold(boxed_vec).unwrap();
    }

    #[test]
    fn fold_simple_boxes_vec() {
        let admin1 = String::from("Admin1");
        let admin2 = String::from("Admin2");
        let alice = String::from("Alice");
        let bob = String::from("Bob"); 
        let allen = String::from("Allen");
        
        let alice_acl = HashSet::from([alice.clone(), admin1.clone(), admin2.clone()]);
        let bob_acl = HashSet::from([bob.clone(), admin1.clone(), admin2.clone()]);
        let allen_acl = HashSet::from([allen.clone(), admin1.clone(), admin2.clone()]);
        
        let mut boxed_vec = Vec::new();
        boxed_vec.extend([BBox::new(100, ACLPolicy::new(alice_acl)), 
                                    BBox::new(99, ACLPolicy::new(bob_acl)), 
                                    BBox::new(95, ACLPolicy::new(allen_acl))]);
        
        let mut unboxed_vec = Vec::new(); 
        unboxed_vec.extend([100, 99, 95]);
            
        //Call to magic_box_fold 
        let agg = magic_box_fold(boxed_vec);
    
        //The aggregated data is as expected
        assert_eq!(agg.as_ref().unwrap().t, unboxed_vec);
    
        // Users are allowed access to aggregated vector as expected  
        let allowed_admin1 = ContextData{ user: admin1.clone()}; 
        let allowed_admin2 = ContextData{ user: admin2.clone()}; 
        assert!(agg.as_ref().unwrap()
                    .policy().check(&allowed_admin1));
        assert!(agg.as_ref().unwrap()
                    .policy().check(&allowed_admin2));

        //and users are disallowed access
        assert!(!agg.as_ref().unwrap()
                        .policy().check(&ContextData{user: String::from("Alice")}));
        assert!(!agg.as_ref().unwrap()
                        .policy().check(&ContextData{user: String::from("Bob") }));
        assert!(!agg.as_ref().unwrap()
                        .policy().check(&ContextData{user: String::from("Allen") }));
        
        println!("Final policy on aggregate: {}", agg.unwrap().policy().name());
    
    }

    #[test]
    fn fold_boxed_struct_vec() {
        let admin = String::from("Admin");
    
        let alice_acl = HashSet::from([String::from("Alice"), admin.clone()]);
        let bob_acl = HashSet::from([String::from("Bob"), admin.clone()]);
        let allen_acl = HashSet::from([String::from("Allen"), admin.clone()]);
    
        let alice_struct = BoxedStruct::new(100, ACLPolicy::new(alice_acl));
        let bob_struct = BoxedStruct::new(95, ACLPolicy::new(bob_acl));
        let allen_struct = BoxedStruct::new(98, ACLPolicy::new(allen_acl));
    
        let mut boxed_vec = Vec::new();
        boxed_vec.extend([alice_struct, bob_struct, allen_struct]);
    
        let mut unboxed_vec = Vec::new(); 
        unboxed_vec.extend([BoxedStructLite::new(100), 
                                  BoxedStructLite::new(95), 
                                  BoxedStructLite::new(98)]);
        //Call to magic_box_fold 
        let agg = magic_box_fold(boxed_vec.clone());

        //The aggregated data is as expected
        assert_eq!(agg.as_ref().unwrap().t, unboxed_vec);

        println!("PolicyAnd on aggregated vector: \n{} \n", agg.as_ref().unwrap().policy().name());

        // Users are allowed and disallowed access to aggregated vector as expected  
        let allowed_admin = ContextData{ user: admin.clone()}; 
        assert!(agg.as_ref().unwrap()
                .policy().check(&allowed_admin));
        assert!(!agg.as_ref().unwrap()
                    .policy().check(&ContextData{user: String::from("Alice")}));
        assert!(!agg.as_ref().unwrap()
                    .policy().check(&ContextData{user: String::from("Bob") }));
        assert!(!agg.as_ref().unwrap()
                    .policy().check(&ContextData{user: String::from("Allen") }));
    }
}