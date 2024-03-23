use std::convert::TryFrom;

use crate::AlohomoraType;
use crate::bbox::BBox;
use crate::policy::{AnyPolicy, NoPolicy, Policy};


// Safe to call from client code because it keeps everything inside a bbox.
pub fn fold<S: AlohomoraType>(s: S) -> Result<BBox<S::Out, AnyPolicy>, ()> {
    let (v, p) = unsafe_fold(s)?;
    Ok(BBox::new(v, p))
}

// Does the folding transformation but without the surrounding bbox at the end.
pub(crate) fn unsafe_fold<S: AlohomoraType>(s: S) -> Result<(S::Out, AnyPolicy), ()> {
    let e = s.to_enum();
    let composed_policy = match e.policy()? {
        None => AnyPolicy::new(NoPolicy {}),
        Some(policy) => policy,
    };
    Ok((S::from_enum(e.remove_bboxes())?, composed_policy))
}

// Fold bbox from outside vector to inside vector.
impl<T, P: Policy + Clone> From<BBox<Vec<T>, P>> for Vec<BBox<T, P>> {
    fn from(x: BBox<Vec<T>, P>) -> Vec<BBox<T, P>> {
        let (t, p) = x.consume();
        t.into_iter().map(|t| BBox::new(t, p.clone())).collect()
    }
}

// Extract bbox from inside vector to outside, use regular fold() for a
// non-failing conversion if vector is empty.
#[derive(Debug)]
pub enum FoldVecError {
    JoinError,
    EmptyVector,
}

impl<T, P: Policy> TryFrom<Vec<BBox<T, P>>> for BBox<Vec<T>, P> {
    type Error = FoldVecError;
    fn try_from(v: Vec<BBox<T, P>>) -> Result<BBox<Vec<T>, P>, Self::Error> {
        let accum = Ok((Vec::new(), None));
        let result = v.into_iter().fold(accum, |accum, e| {
            let (mut v, p) = accum?;
            let (t, ep) = e.consume();
            v.push(t);
            match p {
                None => Ok((v, Some(ep))),
                Some(p) =>
                    match p.join_logic(ep) {
                        Err(_) => Err(FoldVecError::JoinError),
                        Ok(p) => Ok((v, Some(p))),
                    }
            }
        });

        let (v, p) = result?;
        match p {
            None => Err(FoldVecError::EmptyVector),
            Some(p) => Ok(BBox::new(v, p)),
        }
    }
}

// Tests
// TODO(babman): simplify these tests
mod tests {
    use crate::bbox::BBox;
    use crate::policy::{Policy, PolicyAnd, AnyPolicy};
    use crate::testing::TestPolicy;

    use std::any::Any;
    use std::collections::{HashSet, HashMap};
    use crate::r#type::{AlohomoraType, AlohomoraTypeEnum};

    pub struct ContextData {
        pub user: String,
    }
    impl ContextData {
        pub fn get_user(&self) -> &String {
            &self.user
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    pub struct ACLPolicy {
        pub owners: HashSet<String>,
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
            if other.is::<ACLPolicy>() { // Policies are combinable
                let other = other.specialize::<ACLPolicy>().unwrap();
                Ok(AnyPolicy::new(self.join_logic(other)?))
            } else {                    // Policies must be stacked
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

    #[derive(Clone, PartialEq, Debug)]
    pub struct BoxedStruct {
        pub score: BBox<u64, TestPolicy<ACLPolicy>>,
    }

    
    #[derive(PartialEq, Debug)]
    pub struct BoxedStructLite {
        pub score: u64,
    }

    #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
    impl AlohomoraType for BoxedStruct { 
        type Out = BoxedStructLite;
        fn to_enum(self) -> AlohomoraTypeEnum {
            let hashmap = HashMap::from([
                (String::from("score"), self.score.to_enum()),
            ]);
            AlohomoraTypeEnum::Struct(hashmap)
        }
        fn from_enum(e: AlohomoraTypeEnum) -> Result<Self::Out, ()> {
            match e {
                AlohomoraTypeEnum::Struct(mut hashmap) => Ok(Self::Out {
                    score: <u64 as AlohomoraType>::from_enum(hashmap.remove("score").unwrap())?,
                }),
                _ => Err(()),
            }
        }
    }

    #[test]
    fn fold_raw_data() {
        let alice = String::from("Alice");
        let num = 28;
        let deci = 32.0;

        //Call magic_box_fold on unboxed data w/ MagicUnbox impl'd
        let alice_res = super::fold(alice.clone());
        let num_res = super::fold(num.clone());
        let deci_res = super::fold(deci.clone());
        //The aggregated data is as expected
        assert_eq!(alice_res.as_ref().unwrap().data(), &alice);
        assert_eq!(num_res.as_ref().unwrap().data(), &num);
        assert_eq!(deci_res.as_ref().unwrap().data(), &deci);

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
        let agg = super::fold(orig_vec);

        //The aggregated data is as expected
        assert_eq!(agg.as_ref().unwrap().data(), &res_vec);

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
        boxed_vec.extend([BBox::new(100, ACLPolicy { owners: alice_acl }),
            BBox::new(99, ACLPolicy { owners: bob_acl }),
            BBox::new(95, ACLPolicy { owners: allen_acl })]);

        let mut unboxed_vec = Vec::new();
        unboxed_vec.extend([100, 99, 95]);

        // Call to magic_box_fold -> will panic bc Policy join() was unsuccessful
        let _agg = super::fold(boxed_vec).unwrap();
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
        boxed_vec.extend([BBox::new(100, ACLPolicy { owners: alice_acl }),
            BBox::new(99, ACLPolicy { owners: bob_acl }),
            BBox::new(95, ACLPolicy { owners: allen_acl })]);

        let mut unboxed_vec = Vec::new();
        unboxed_vec.extend([100, 99, 95]);

        //Call to magic_box_fold
        let agg = super::fold(boxed_vec);

        //The aggregated data is as expected
        assert_eq!(agg.as_ref().unwrap().data(), &unboxed_vec);

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

        let alice_struct = BoxedStruct { score: BBox::new(100, ACLPolicy { owners: alice_acl }.into()) };
        let bob_struct = BoxedStruct { score: BBox::new(95, ACLPolicy { owners: bob_acl }.into()) };
        let allen_struct = BoxedStruct { score: BBox::new(98, ACLPolicy { owners: allen_acl }.into()) };

        let mut boxed_vec = Vec::new();
        boxed_vec.extend([alice_struct, bob_struct, allen_struct]);

        let mut unboxed_vec = Vec::new();
        unboxed_vec.extend([BoxedStructLite { score: 100 },
            BoxedStructLite { score: 95 },
            BoxedStructLite { score: 98 }]);
        //Call to magic_box_fold
        let agg = super::fold(boxed_vec.clone());

        //The aggregated data is as expected
        assert_eq!(agg.as_ref().unwrap().data(), &unboxed_vec);

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

    #[test]
    fn special_case_fold_vector() {
        use std::convert::TryInto;
        use super::FoldVecError;

        let vec = vec![String::from("A"), String::from("B"), String::from("C")];
        let owners = HashSet::from([String::from("Alice"), String::from("Bob"), String::from("Carl")]);
        let policy = TestPolicy::new(ACLPolicy { owners });
        let bbox = BBox::new(vec, policy.clone());

        // fold box into vector.
        let vec: Vec<BBox<String, TestPolicy<ACLPolicy>>> = bbox.into();
        assert_eq!(vec.len(), 3);
        assert_eq!(vec[0].policy(), &policy);
        assert_eq!(vec[1].policy(), &policy);
        assert_eq!(vec[2].policy(), &policy);
        assert_eq!(vec[0].clone().discard_box(), "A");
        assert_eq!(vec[1].clone().discard_box(), "B");
        assert_eq!(vec[2].clone().discard_box(), "C");

        // fold box out of vector.
        let mut vec = Vec::new();
        vec.push(BBox::new(String::from("A"), TestPolicy::new(ACLPolicy { owners: HashSet::from([String::from("Alice"), String::from("Admin")]) })));
        vec.push(BBox::new(String::from("B"), TestPolicy::new(ACLPolicy { owners: HashSet::from([String::from("Bob"), String::from("Admin")]) })));
        vec.push(BBox::new(String::from("C"), TestPolicy::new(ACLPolicy { owners: HashSet::from([String::from("Carl"), String::from("Admin")]) })));

        let bbox: Result<BBox<Vec<String>, TestPolicy<ACLPolicy>>, FoldVecError> = vec.try_into();
        println!("{:?}", bbox);
        assert!(bbox.is_ok());
        if let Ok(bbox) = bbox {
            assert_eq!(bbox.policy(), &TestPolicy::new(ACLPolicy { owners: HashSet::from([String::from("Admin")]) }));
            assert_eq!(bbox.discard_box(), vec![String::from("A"), String::from("B"), String::from("C")]);
        }

        // fold box out of vector but with a join error due to non-overlapping ACLs.
        let mut vec = Vec::new();
        vec.push(BBox::new(String::from("A"), TestPolicy::new(ACLPolicy { owners: HashSet::from([String::from("Alice")]) })));
        vec.push(BBox::new(String::from("B"), TestPolicy::new(ACLPolicy { owners: HashSet::from([String::from("Bob")]) })));
        vec.push(BBox::new(String::from("C"), TestPolicy::new(ACLPolicy { owners: HashSet::from([String::from("Carl")]) })));

        let bbox: Result<BBox<Vec<String>, TestPolicy<ACLPolicy>>, FoldVecError> = vec.try_into();
        assert!(bbox.is_err());
        if let Err(error) = bbox {
            assert!(matches!(error, FoldVecError::JoinError));
        }

        // fold box out of empty vector.
        let vec = Vec::new();
        let bbox: Result<BBox<Vec<String>, TestPolicy<ACLPolicy>>, FoldVecError> = vec.try_into();
        assert!(bbox.is_err());
        if let Err(error) = bbox {
            assert!(matches!(error, FoldVecError::EmptyVector));
        }
    }
}