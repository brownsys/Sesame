use std::any::Any;
use crate::bbox::BBox;
use crate::policy::{AnyPolicyDyn, AnyPolicyable, NoPolicy, OptionPolicy, Policy, PolicyDyn, PolicyDynRelation};
use crate::sesame_type_dyns::{SesameDyn, SesameDynRelation};
use crate::SesameType;

pub fn fold<P: PolicyDyn + ?Sized, S: SesameType<dyn Any, P>>(s: S) -> Result<BBox<S::Out, AnyPolicyDyn<P>>, ()> {
    let (v, p) = Foldable::unsafe_fold(s)?;
    Ok(BBox::new(v, p))
}

// Private trait that implements folding out nested BBoxes.
pub(crate) trait Foldable<P: PolicyDyn + ?Sized>: SesameType<dyn Any, P> {
    fn unsafe_fold(self) -> Result<(Self::Out, AnyPolicyDyn<P>), ()>
    where
        Self: Sized;
}

// The general, unoptimized implementation of folding that works for all `AlohomoraType` types.
// It's marked with the `default` keyword so we can override it with optimized implementations for specific types.
impl<P: PolicyDyn + ?Sized, T: SesameType<dyn Any, P>> Foldable<P> for T {
    default fn unsafe_fold(self) -> Result<(T::Out, AnyPolicyDyn<P>), ()>
    where
        Self: Sized,
    {
        let e = self.to_enum();
        let (t, p) = e.remove_bboxes2();
        let composed_policy = match p? {
            None => todo!("NoPolicy"),
            Some(policy) => policy,
        };
        Ok((Self::from_enum(t)?, composed_policy))
    }
}

// A more optimized version of unwrap for a simple vec of BBoxes.
impl<T: Any, P: AnyPolicyable, PDyn: PolicyDyn + ?Sized + PolicyDynRelation<P>> Foldable<PDyn> for Vec<BBox<T, P>> {
    fn unsafe_fold(self) -> Result<(Self::Out, AnyPolicyDyn<PDyn>), ()>
    where
        Self: Sized,
    {
        let accum = (Vec::with_capacity(self.len()), OptionPolicy::NoPolicy);
        let (v, p) = self.into_iter().fold(accum, |accum, e| {
            let (mut v, p) = accum;
            let (t, ep) = e.consume();
            v.push(t);
            match p {
                OptionPolicy::NoPolicy => (v, OptionPolicy::Policy(ep)),
                OptionPolicy::Policy(p) => match p.join_logic(ep) {
                    Err(_) => panic!("Cannot unsafe_fold vector [opt]; unsatisfiable policy"),
                    Ok(p) => (v, OptionPolicy::Policy(p)),
                },
            }
        });

        match p {
            OptionPolicy::NoPolicy => Ok((v, AnyPolicyDyn::new(todo!("NoPolicy")))),
            OptionPolicy::Policy(p) => Ok((v, AnyPolicyDyn::new(p))),
        }
    }
}

// Expands to code that optimizes folding for simple vecs with tuples of bboxes. -- Eg. `Vec<(BBox<T, P>,)>`
macro_rules! optimized_tup_fold {
    ($([$A:tt,$P:tt]),*) => (
        impl<$($A: Any,)* $($P: AnyPolicyable,)* PDyn: PolicyDyn + ?Sized> Foldable<PDyn> for Vec<($(BBox<$A, $P>,)*)>
        where $(PDyn: PolicyDynRelation<$P>,)* {
            fn unsafe_fold(self) -> Result<(Self::Out, AnyPolicyDyn<PDyn>), ()> where Self: Sized {
                let mut v: Vec<($($A,)*)> = Vec::with_capacity(self.len());
                // TODO(babman): same issue with NoPolicy.
                let mut p: AnyPolicyDyn<PDyn> = todo!("NoPolicy");
                for tup in self {
                    #[allow(non_snake_case)]
                    let ($($A,)*) = tup;
                    #[allow(non_snake_case)]
                    let ($(($A, $P),)*) = ($($A.consume(),)*);

                    v.push(($($A,)*));
                    // TODO(babman): same issue with join.
                    /*
                    p = p.join(
                        IntoIterator::into_iter([$(AnyPolicyDyn::new($P),)*]).fold(
                            AnyPolicyDyn::new(NoPolicy {})
                            |p: AnyPolicyDyn<PDyn>, ep: AnyPolicyDyn<PDyn>| {
                                p.join(ep).expect("Cannot fold vector in; unsatisfiable policy")
                            }
                        )
                    )?;
                     */
                }
                Ok((v, p))
            }
        }
    );
}

optimized_tup_fold!([T1, P1]);
optimized_tup_fold!([T1, P1], [T2, P2]);
optimized_tup_fold!([T1, P1], [T2, P2], [T3, P3]);
optimized_tup_fold!([T1, P1], [T2, P2], [T3, P3], [T4, P4]);
optimized_tup_fold!([T1, P1], [T2, P2], [T3, P3], [T4, P4], [T5, P5]);
optimized_tup_fold!([T1, P1], [T2, P2], [T3, P3], [T4, P4], [T5, P5], [T6, P6]);
optimized_tup_fold!(
    [T1, P1],
    [T2, P2],
    [T3, P3],
    [T4, P4],
    [T5, P5],
    [T6, P6],
    [T7, P7]
);
optimized_tup_fold!(
    [T1, P1],
    [T2, P2],
    [T3, P3],
    [T4, P4],
    [T5, P5],
    [T6, P6],
    [T7, P7],
    [T8, P8]
);

// Fold bbox from inside vector to the outside. Same as generic fold(...) but preserves policy type.
impl<T, P: AnyPolicyable> From<Vec<BBox<T, P>>> for BBox<Vec<T>, OptionPolicy<P>> {
    fn from(v: Vec<BBox<T, P>>) -> BBox<Vec<T>, OptionPolicy<P>> {
        let accum = (Vec::new(), OptionPolicy::NoPolicy);
        let result = v.into_iter().fold(accum, |accum, e| {
            let (mut v, p) = accum;
            let (t, ep) = e.consume();
            v.push(t);
            match p {
                OptionPolicy::NoPolicy => (v, OptionPolicy::Policy(ep)),
                OptionPolicy::Policy(p) => match p.join_logic(ep) {
                    Err(_) => panic!("Cannot fold vector in; unsatisfiable policy"),
                    Ok(p) => (v, OptionPolicy::Policy(p)),
                },
            }
        });

        let (v, p) = result;
        BBox::new(v, p)
    }
}

// Tests
#[cfg(test)]
mod tests {
    use crate::bbox::BBox;
    use crate::policy::{AnyPolicyBB, OptionPolicy, Policy, PolicyAnd, Reason};
    use crate::{SesameType, SesameTypeEnum};
    use crate::testing::TestPolicy;

    use crate::context::UnprotectedContext;
    use std::any::Any;
    use std::collections::{HashMap, HashSet};
    use std::iter::FromIterator;

    #[derive(Clone, PartialEq, Debug)]
    pub struct ACLPolicy {
        pub owners: HashSet<u32>,
    }
    impl ACLPolicy {
        pub fn new(x: &[u32]) -> ACLPolicy {
            ACLPolicy {
                owners: HashSet::from_iter(x.iter().cloned()),
            }
        }
    }
    impl Policy for ACLPolicy {
        fn name(&self) -> String {
            format!("ACLPolicy(owners: {:?})", self.owners)
        }
        fn check(&self, _context: &UnprotectedContext, _reason: Reason) -> bool {
            true
        }
        fn join(&self, other: AnyPolicyBB) -> Result<AnyPolicyBB, ()> {
            if other.is::<ACLPolicy>() {
                let other = other.specialize::<ACLPolicy>().unwrap();
                Ok(AnyPolicyBB::new(self.join_logic(other)?))
            } else {
                Ok(AnyPolicyBB::new(PolicyAnd::new(self.clone(), other)))
            }
        }
        fn join_logic(&self, other: Self) -> Result<Self, ()> {
            let intersection: HashSet<_> = self
                .owners
                .intersection(&other.owners)
                .map(|owner| owner.clone())
                .collect();
            if intersection.len() > 0 {
                Ok(ACLPolicy {
                    owners: intersection,
                })
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
    impl SesameType for BoxedStruct {
        type Out = BoxedStructLite;
        fn to_enum(self) -> SesameTypeEnum {
            let hashmap = HashMap::from([
                (String::from("x"), self.x.to_enum()),
                (String::from("y"), self.y.to_enum()),
                (String::from("z"), self.z.to_enum()),
            ]);
            SesameTypeEnum::Struct(hashmap)
        }
        fn from_enum(e: SesameTypeEnum) -> Result<Self::Out, ()> {
            match e {
                SesameTypeEnum::Struct(mut hashmap) => Ok(Self::Out {
                    x: BBox::<u64, TestPolicy<ACLPolicy>>::from_enum(hashmap.remove("x").unwrap())?,
                    y: BBox::<String, TestPolicy<ACLPolicy>>::from_enum(
                        hashmap.remove("y").unwrap(),
                    )?,
                    z: String::from_enum(hashmap.remove("z").unwrap())?,
                }),
                _ => Err(()),
            }
        }
    }

    #[test]
    fn test_join_policies() {
        let policy1 = TestPolicy::new(ACLPolicy::new(&[10, 20]));
        let policy2 = TestPolicy::new(ACLPolicy::new(&[10, 30]));
        let joined = policy1.join(AnyPolicyBB::new(policy2)).unwrap();
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
            z: String::from("bye"),
        };

        let bbox = super::fold(boxed_struct).unwrap();
        let bbox = bbox.specialize_policy::<TestPolicy<ACLPolicy>>().unwrap();
        assert_eq!(bbox.policy().policy().owners, HashSet::from_iter([10]));
        assert_eq!(
            bbox.discard_box(),
            BoxedStructLite {
                x: 1,
                y: String::from("hello"),
                z: String::from("bye"),
            }
        );
    }

    #[test]
    #[should_panic]
    fn test_fold_struct_unsat() {
        let policy1 = TestPolicy::new(ACLPolicy::new(&[10, 20]));
        let policy2 = TestPolicy::new(ACLPolicy::new(&[40, 30]));

        let boxed_struct = BoxedStruct {
            x: BBox::new(1, policy1),
            y: BBox::new(String::from("hello"), policy2),
            z: String::from("bye"),
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

        let bbox: BBox<_, AnyPolicyBB> = super::fold(vec).unwrap();
        let bbox = bbox.specialize_policy::<TestPolicy<ACLPolicy>>().unwrap();
        assert_eq!(bbox.policy().policy().owners, HashSet::from_iter([40]));
        assert_eq!(bbox.clone().discard_box(), vec![10, 20, 30]);

        // inverse fold for vector.
        let vec: Vec<BBox<i32, TestPolicy<ACLPolicy>>> = bbox.fold_in();
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

        let _: BBox<_, AnyPolicyBB> = super::fold(vec).unwrap();
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
        let bbox = bbox.specialize_policy::<TestPolicy<ACLPolicy>>().unwrap();
        assert_eq!(bbox.policy().policy().owners, HashSet::from_iter([40]));
        assert_eq!(
            bbox.discard_box(),
            vec![
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
            ]
        );
    }
}
