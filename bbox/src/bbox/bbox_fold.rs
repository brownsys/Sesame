use crate::bbox::BBox;
use crate::policy::Policy;
use std::convert::TryFrom;

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
