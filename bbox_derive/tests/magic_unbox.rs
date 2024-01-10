use bbox::policy::{NoPolicy};
use bbox::bbox::{BBox, MagicUnbox, MagicUnboxEnum};
use bbox_derive::{BBoxRender, FromBBoxForm, get, post, MagicUnbox};

use serde::Serialize; 
use std::any::Any;

/* Testing To-Dos
    Fields with classic boxes
    Fields without bboxes
    Fields with path segment before BBox
    Vec<BBox<String, NoPolicy>>

    Can we do the to_enum and from_enum operations? Call magic_box_fold for downstream?
    Can we operate on the derived Lite struct as expected?
*/

#[derive(MagicUnbox)]
pub struct SingleField {
    f1: String
}

#[derive(MagicUnbox)]
pub struct NoBoxes {
    f1: u64, 
    f2: String,
}

#[derive(MagicUnbox)]
pub struct SimpleBoxed {
    f1: BBox<u64, NoPolicy>, 
    f2: BBox<String, NoPolicy>, 
}

#[derive(MagicUnbox)]
pub struct MixedBoxed {
    f1: BBox<u64, NoPolicy>, 
    f2: String, 
}

#[derive(MagicUnbox)]
pub struct VecBoxed {
    f1: Vec<BBox<u64, NoPolicy>>,
}


#[test]
fn struct_exists() {

}

fn to_enum_correct() {

}

fn from_enum_correct() {

}

