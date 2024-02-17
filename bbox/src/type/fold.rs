use crate::bbox::BBox;
use crate::policy::{AnyPolicy, NoPolicy, Policy};
use crate::r#type::{AlohomoraType, AlohomoraTypePolicy};

// Safe to call from client code because it keeps everything inside a bbox.
pub fn fold<S: AlohomoraType>(s: S) -> Result<BBox<S::Out, AnyPolicy>, ()> {
    let (v, p) = unsafe_fold(s)?;
    Ok(BBox::new(v, p))
}

// Does the folding transformation but without the surrounding bbox at the end.
pub(crate) fn unsafe_fold<S: AlohomoraType>(s: S) -> Result<(S::Out, AnyPolicy), ()> {
    let e = s.to_enum();
    let composed_policy = match e.policy() {
        AlohomoraTypePolicy::NoPolicy => AnyPolicy::new(NoPolicy {}),
        AlohomoraTypePolicy::Policy(policy) => policy,
    };
    Ok((S::from_enum(e.remove_bboxes())?, composed_policy))
}

// Fold policy from outside vector to inside vector.
impl<T, P: Policy + Clone> From<BBox<Vec<T>, P>> for Vec<BBox<T, P>> {
    fn from(x: BBox<Vec<T>, P>) -> Vec<BBox<T, P>> {
        let p = x.p;
        x.t.into_iter().map(|t| BBox::new(t, p.clone())).collect()
    }
}
impl<T, P: Policy + Clone + 'static> From<Vec<BBox<T, P>>> for BBox<Vec<T>, AnyPolicy> {
    fn from(v: Vec<BBox<T, P>>) -> BBox<Vec<T>, AnyPolicy> {
        v.into_iter().fold(
            BBox::new(Vec::new(), AnyPolicy::new(NoPolicy {})),
            |mut acc, e| {
                acc.t.push(e.t);
                acc.p = acc.p.join(AnyPolicy::new(e.p)).unwrap();
                acc
        })
    }
}

// Tests

mod tests {
    use crate::policy::{Policy, PolicyAnd, AnyPolicy};
    use crate::bbox::BBox;

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

    #[derive(Clone, PartialEq)]
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

    #[derive(Clone, PartialEq, Debug)]
    pub struct BoxedStruct {
        pub score: BBox<u64, ACLPolicy>,
    }

    #[derive(PartialEq, Debug)]
    pub struct BoxedStructLite {
        pub score: u64,
    }
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
        let agg = super::fold(orig_vec);

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
        boxed_vec.extend([BBox::new(100, ACLPolicy { owners: alice_acl }),
            BBox::new(99, ACLPolicy { owners: bob_acl }),
            BBox::new(95, ACLPolicy { owners: allen_acl })]);

        let mut unboxed_vec = Vec::new();
        unboxed_vec.extend([100, 99, 95]);

        //Call to magic_box_fold -> will panic bc Policy join() was unsuccessful
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

        let alice_struct = BoxedStruct { score: BBox::new(100, ACLPolicy { owners: alice_acl }) };
        let bob_struct = BoxedStruct { score: BBox::new(95, ACLPolicy { owners: bob_acl }) };
        let allen_struct = BoxedStruct { score: BBox::new(98, ACLPolicy { owners: allen_acl }) };

        let mut boxed_vec = Vec::new();
        boxed_vec.extend([alice_struct, bob_struct, allen_struct]);

        let mut unboxed_vec = Vec::new();
        unboxed_vec.extend([BoxedStructLite { score: 100 },
            BoxedStructLite { score: 95 },
            BoxedStructLite { score: 98 }]);
        //Call to magic_box_fold
        let agg = super::fold(boxed_vec.clone());

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