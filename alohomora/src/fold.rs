use crate::{AlohomoraType, Unwrapper};
use crate::bbox::BBox;
use crate::policy::{AnyPolicy, NoPolicy, OptionPolicy, Policy};

pub fn fold<S: AlohomoraType>(s: S) -> Result<BBox<S::Out, S::Policy>, ()> {
    let (t, p) = s.inner_fold(&Unwrapper::new())?;
    Ok(BBox::new(t, p))
}

// Fold bbox from outside vector to inside vector.
// TODO(babman): mark this as fold in.
impl<T, P: Policy + Clone> From<BBox<Vec<T>, P>> for Vec<BBox<T, P>> {
    fn from(x: BBox<Vec<T>, P>) -> Vec<BBox<T, P>> {
        let (t, p) = x.consume();
        t.into_iter().map(|t| BBox::new(t, p.clone())).collect()
    }
}

// Fold bbox from inside vector to the outside. Same as generic fold(...) but preserves policy type.
impl<T, P: Policy + Clone> From<Vec<BBox<T, P>>> for BBox<Vec<T>, OptionPolicy<P>> {
    fn from(v: Vec<BBox<T, P>>) -> BBox<Vec<T>, OptionPolicy<P>> {
        fold(v).expect("Cannot fold vector out; unsatisfiable policy")
    }
}

// Tests
#[cfg(test)]
mod tests {
    use crate::r#type::AlohomoraType;
    use crate::bbox::BBox;
    use crate::policy::{Policy, PolicyAnd, AnyPolicy, OptionPolicy, Reason};
    use crate::testing::TestPolicy;

    use std::collections::{HashSet, HashMap};
    use std::iter::FromIterator;
    use crate::context::UnprotectedContext;
    use crate::Unwrapper;

    #[derive(Clone, PartialEq, Debug)]
    pub struct ACLPolicy {
        pub owners: HashSet<u32>,
    }
    impl ACLPolicy {
        pub fn new(x: &[u32]) -> ACLPolicy {
            ACLPolicy { owners: HashSet::from_iter(x.iter().cloned()) }
        }
    }
    impl Policy for ACLPolicy {
        fn name(&self) -> String {
            format!("ACLPolicy(owners: {:?})", self.owners)
        }
        fn check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool { true }
        fn join(&self, other: AnyPolicy) -> Result<AnyPolicy, ()> {
            if other.is::<ACLPolicy>() {
                let other = other.specialize::<ACLPolicy>().unwrap();
                Ok(AnyPolicy::new(self.join_logic(other)?))
            } else {
                Ok(AnyPolicy::new(PolicyAnd::new(self.clone(), other)))
            }
        }
        fn join_logic(&self, other: Self) -> Result<Self, ()> {
            let intersection: HashSet<_> =
                self.owners.intersection(&other.owners)
                    .map(|owner| owner.clone())
                    .collect();
            if intersection.len() > 0 {
                Ok(ACLPolicy { owners: intersection })
            } else {
                Err(())
            }
        }
    }

    #[derive(Clone, PartialEq, Debug)]
    pub struct BoxedStruct {
        pub x: BBox<u64, TestPolicy<ACLPolicy>>,
        pub y: BBox<String, TestPolicy<ACLPolicy>>,
        pub z: String,
    }


    #[derive(PartialEq, Debug)]
    pub struct BoxedStructLite {
        pub x: u64,
        pub y: String,
        pub z: String,
    }

    #[doc = "Library implementation of AlohomoraType. Do not copy this docstring!"]
    impl AlohomoraType for BoxedStruct {
        type Out = BoxedStructLite;
        type Policy = AnyPolicy;
        fn inner_fold(self, unwrapper: &Unwrapper) -> Result<(Self::Out, Self::Policy), ()> {
            let (t_x, p_x) = self.x.inner_fold(unwrapper)?;
            let (t_y, p_y) = self.y.inner_fold(unwrapper)?;
            let (t_z, p_z) = self.z.inner_fold(unwrapper)?;

            let p = p_x.join(AnyPolicy::new(p_y))?;
            Ok((
                BoxedStructLite {
                    x: t_x,
                    y: t_y,
                    z: t_z,
                },
                p
            ))
        }
    }

    #[test]
    fn test_join_policies() {
        let policy1 = TestPolicy::new(ACLPolicy::new(&[10, 20]));
        let policy2 = TestPolicy::new(ACLPolicy::new(&[10, 30]));
        let joined = policy1.join(AnyPolicy::new(policy2)).unwrap();
        let joined: TestPolicy<ACLPolicy> = joined.specialize().unwrap();
        assert_eq!(joined.policy().owners, HashSet::from_iter([10]));
    }

    #[test]
    fn test_fold_struct() {
        let policy1 = TestPolicy::new(ACLPolicy::new(&[10, 20]));
        let policy2 = TestPolicy::new(ACLPolicy::new(&[10, 30]));

        let boxed_struct = BoxedStruct {
            x: BBox::new(1, policy1),
            y: BBox::new(String::from("hello"), policy2),
            z: String::from("bye")
        };

        let bbox = super::fold(boxed_struct).unwrap();
        let bbox = bbox.specialize_policy::<TestPolicy<ACLPolicy>>().unwrap();
        assert_eq!(bbox.policy().policy().owners, HashSet::from_iter([10]));
        assert_eq!(bbox.discard_box(), BoxedStructLite {
            x: 1,
            y: String::from("hello"),
            z: String::from("bye"),
        });
    }

    #[test]
    #[should_panic]
    fn test_fold_struct_unsat() {
        let policy1 = TestPolicy::new(ACLPolicy::new(&[10, 20]));
        let policy2 = TestPolicy::new(ACLPolicy::new(&[40, 30]));

        let boxed_struct = BoxedStruct {
            x: BBox::new(1, policy1),
            y: BBox::new(String::from("hello"), policy2),
            z: String::from("bye")
        };

        let _ = super::fold(boxed_struct).unwrap();
    }

    #[test]
    fn test_fold_vec() {
        let policy1 = TestPolicy::new(ACLPolicy::new(&[10, 20, 40]));
        let policy2 = TestPolicy::new(ACLPolicy::new(&[10, 30, 40]));
        let policy3 = TestPolicy::new(ACLPolicy::new(&[20, 30, 40]));

        let vec = vec![
            BBox::new(10, policy1),
            BBox::new(20, policy2),
            BBox::new(30, policy3),
        ];

        let bbox = super::fold(vec).unwrap();
        let bbox = bbox.to_some_policy();
        assert_eq!(bbox.policy().policy().owners, HashSet::from_iter([40]));
        assert_eq!(bbox.clone().discard_box(), vec![10, 20, 30]);

        // inverse fold for vector.
        let vec: Vec<BBox<i32, TestPolicy<ACLPolicy>>> = Vec::from(bbox);
        assert_eq!(vec[0].policy().policy().owners, HashSet::from_iter([40]));
        assert_eq!(vec[1].policy().policy().owners, HashSet::from_iter([40]));
        assert_eq!(vec[2].policy().policy().owners, HashSet::from_iter([40]));
        assert_eq!(vec[0].clone().discard_box(), 10);
        assert_eq!(vec[1].clone().discard_box(), 20);
        assert_eq!(vec[2].clone().discard_box(), 30);
    }

    #[test]
    fn test_fold_vec_special_case() {
        let policy1 = TestPolicy::new(ACLPolicy::new(&[10, 20, 40]));
        let policy2 = TestPolicy::new(ACLPolicy::new(&[10, 30, 40]));
        let policy3 = TestPolicy::new(ACLPolicy::new(&[20, 30, 40]));

        let vec = vec![
            BBox::new(10, policy1),
            BBox::new(20, policy2),
            BBox::new(30, policy3),
        ];

        let bbox: BBox<Vec<i32>, OptionPolicy<TestPolicy<ACLPolicy>>> = BBox::from(vec);
        let bbox = bbox.specialize().right().unwrap();
        assert_eq!(bbox.policy().policy().owners, HashSet::from_iter([40]));
        assert_eq!(bbox.clone().discard_box(), vec![10, 20, 30]);
    }

    #[test]
    #[should_panic]
    fn test_fold_vec_unsat() {
        let policy1 = TestPolicy::new(ACLPolicy::new(&[10, 20, 40]));
        let policy2 = TestPolicy::new(ACLPolicy::new(&[10, 30, 40]));
        let policy3 = TestPolicy::new(ACLPolicy::new(&[20, 30, 50]));

        let vec = vec![
            BBox::new(10, policy1),
            BBox::new(20, policy2),
            BBox::new(30, policy3),
        ];

        let _ = super::fold(vec).unwrap();
    }

    #[test]
    fn test_fold_vec_struct() {
        let policy1 = TestPolicy::new(ACLPolicy::new(&[10, 20, 40]));
        let policy2 = TestPolicy::new(ACLPolicy::new(&[10, 30, 40]));
        let policy3 = TestPolicy::new(ACLPolicy::new(&[20, 30, 40]));

        let vec = vec![
            BoxedStruct {
                x: BBox::new(10, policy1.clone()),
                y: BBox::new(String::from("x0"), policy2.clone()),
                z: String::from("z0"),
            },
            BoxedStruct {
                x: BBox::new(20, policy3.clone()),
                y: BBox::new(String::from("x1"), policy1.clone()),
                z: String::from("z1"),
            },
            BoxedStruct {
                x: BBox::new(100, policy2.clone()),
                y: BBox::new(String::from("x2"), policy3.clone()),
                z: String::from("z2"),
            },
        ];

        let bbox = super::fold(vec).unwrap();
        let bbox = bbox.to_some_policy().specialize_policy::<TestPolicy<ACLPolicy>>().unwrap();
        assert_eq!(bbox.policy().policy().owners, HashSet::from_iter([40]));
        assert_eq!(bbox.discard_box(), vec![
            BoxedStructLite {
                x: 10,
                y: String::from("x0"),
                z: String::from("z0"),
            },
            BoxedStructLite {
                x: 20,
                y: String::from("x1"),
                z: String::from("z1"),
            },
            BoxedStructLite {
                x: 100,
                y: String::from("x2"),
                z: String::from("z2"),
            },
        ]);
    }
}
