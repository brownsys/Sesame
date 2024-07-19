use alohomora::policy::{AnyPolicy, NoPolicy};
use alohomora::bbox::BBox;
use alohomora::AlohomoraType;
use std::collections::HashMap;
use alohomora::fold::fold;
use alohomora_derive::AlohomoraType;

// The struct is its own out type.
#[derive(AlohomoraType, Clone, PartialEq, Debug)]
#[alohomora_out_type(to_derive = [PartialEq, Debug])]
pub struct NoBoxes {
    pub f1: u64,
    pub f2: String,
}
#[test]
fn test_derived_no_boxes() {
    let input = NoBoxes { f1: 10, f2: String::from("hello") };
    let folded: BBox<_, AnyPolicy> = fold(input).unwrap();
    let folded: BBox<_, NoPolicy> = folded.specialize_policy().unwrap();
    let folded = folded.discard_box();
    assert_eq!(folded.f1, 10);
    assert_eq!(folded.f2, String::from("hello"));
}

// The struct contains a mix.
#[derive(AlohomoraType, Clone, PartialEq, Debug)]
#[alohomora_out_type(to_derive = [PartialEq, Debug])]
pub struct MixedBoxes {
    pub f1: BBox<u64, NoPolicy>,
    pub f2: BBox<String, NoPolicy>,
    pub f3: u64,
    pub f4: String,
}

#[test]
fn test_mixed_boxes() {
    let input = MixedBoxes {
        f1: BBox::new(10, NoPolicy {}),
        f2: BBox::new(String::from("hello"), NoPolicy {}),
        f3: 20,
        f4: String::from("bye"),
    };

    type MixedBoxesOut = <MixedBoxes as AlohomoraType>::Out;
    let expected = MixedBoxesOut {
        f1: 10,
        f2: String::from("hello"),
        f3: 20,
        f4: String::from("bye"),
    };

    let folded: BBox<<MixedBoxes as AlohomoraType>::Out, AnyPolicy> = fold(input).unwrap();
    let folded: BBox<<MixedBoxes as AlohomoraType>::Out, NoPolicy> = folded.specialize_policy().unwrap();
    assert_eq!(folded.discard_box(), expected);
}

// Test specifying the output name.
// Test having containers of nested types.
#[derive(AlohomoraType, Clone, PartialEq, Debug)]
#[alohomora_out_type(name = NestedOut, to_derive = [PartialEq, Debug])]
pub struct NestedBoxes {
    pub f1: NoBoxes,
    pub f2: Vec<MixedBoxes>,
    pub f3: HashMap<String, MixedBoxes>,
}

// Implement some additional methods on out type.
impl NestedOut {
    pub fn sum(self) -> u64 {
        let v_sum = self.f2.iter().fold(0, |acc, v| acc + v.f1 + v.f3);
        let m_sum =  self.f3.iter().fold(0, |acc, (_k, v)| acc + v.f1 + v.f3);
        self.f1.f1 + v_sum + m_sum
    }
}


#[test]
fn test_nested_boxes() {
    let input = NestedBoxes {
        f1: NoBoxes {
            f1: 10,
            f2: String::from("hello"),
        },
        f2: vec![
            MixedBoxes {
                f1: BBox::new(1, NoPolicy {}),
                f2: BBox::new(String::from("v0.f2"), NoPolicy {}),
                f3: 2,
                f4: String::from("v0.f4"),
            },
            MixedBoxes {
                f1: BBox::new(3, NoPolicy {}),
                f2: BBox::new(String::from("v1.f2"), NoPolicy {}),
                f3: 4,
                f4: String::from("v1.f4"),
            },
            MixedBoxes {
                f1: BBox::new(5, NoPolicy {}),
                f2: BBox::new(String::from("v2.f2"), NoPolicy {}),
                f3: 6,
                f4: String::from("v2.f4"),
            },
        ],
        f3: HashMap::from([
            (String::from("k0"), MixedBoxes {
                f1: BBox::new(7, NoPolicy {}),
                f2: BBox::new(String::from("k0.f2"), NoPolicy {}),
                f3: 8,
                f4: String::from("k0.f4"),
            }),
            (String::from("k1"), MixedBoxes {
                f1: BBox::new(9, NoPolicy {}),
                f2: BBox::new(String::from("k1.f2"), NoPolicy {}),
                f3: 10,
                f4: String::from("k1.f4"),
            })
        ]),
    };

    let folded: BBox<NestedOut, AnyPolicy> = fold(input).unwrap();
    let folded: BBox<NestedOut, NoPolicy>  = folded.specialize_policy().unwrap();
    let folded: NestedOut = folded.discard_box();


    assert_eq!(folded, NestedOut {
        f1: NoBoxesOut {
            f1: 10,
            f2: String::from("hello"),
        },
        f2: vec![
            MixedBoxesOut {
                f1: 1,
                f2: String::from("v0.f2"),
                f3: 2,
                f4: String::from("v0.f4"),
            },
            MixedBoxesOut {
                f1: 3,
                f2: String::from("v1.f2"),
                f3: 4,
                f4: String::from("v1.f4"),
            },
            MixedBoxesOut {
                f1: 5,
                f2: String::from("v2.f2"),
                f3: 6,
                f4: String::from("v2.f4"),
            },
        ],
        f3: HashMap::from([
            (String::from("k0"), MixedBoxesOut {
                f1: 7,
                f2: String::from("k0.f2"),
                f3: 8,
                f4: String::from("k0.f4"),
            }),
            (String::from("k1"), MixedBoxesOut {
                f1: 9,
                f2: String::from("k1.f2"),
                f3: 10,
                f4: String::from("k1.f4"),
            })
        ]),
    });
    assert_eq!(folded.sum(), 65);
}


// Struct with verbatim items.
#[derive(PartialEq, Debug, Clone)]
pub struct VerbatimType(pub u32, pub String);

#[derive(AlohomoraType, Clone, PartialEq, Debug)]
#[alohomora_out_type(to_derive = [PartialEq, Debug])]
#[alohomora_out_type(verbatim = [f3])]
pub struct VerbatimBox {
    pub f1: u64,
    pub f2: BBox<String, NoPolicy>,
    pub f3: VerbatimType,
}

#[test]
fn test_derived_verbatim() {
    let input = VerbatimBox {
        f1: 10,
        f2: BBox::new(String::from("hello"), NoPolicy {}),
        f3: VerbatimType(20, String::from("bye")),
    };

    let folded: BBox<_, AnyPolicy> = fold(input).unwrap();
    let folded: BBox<_, NoPolicy> = folded.specialize_policy().unwrap();
    let folded = folded.discard_box();

    assert_eq!(folded.f1, 10);
    assert_eq!(folded.f2, String::from("hello"));
    assert_eq!(folded.f3, VerbatimType(20, String::from("bye")));
}


// Struct with only verbatim items.
#[derive(AlohomoraType, Clone, PartialEq, Debug)]
#[alohomora_out_type(to_derive = [PartialEq, Debug])]
#[alohomora_out_type(verbatim = [f1, f2])]
pub struct OnlyVerbatimBox {
    pub f1: u64,
    pub f2: VerbatimType,
}

#[test]
fn test_derived_only_verbatim() {
    let input = OnlyVerbatimBox {
        f1: 10,
        f2: VerbatimType(20, String::from("bye")),
    };

    let folded: BBox<_, AnyPolicy> = fold(input).unwrap();
    let folded: BBox<_, NoPolicy> = folded.specialize_policy().unwrap();
    let folded = folded.discard_box();

    assert_eq!(folded.f1, 10);
    assert_eq!(folded.f2, VerbatimType(20, String::from("bye")));
}
