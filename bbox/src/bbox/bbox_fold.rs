use crate::bbox::BBox;
use crate::policy::{Policy, Conjunction};
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

pub fn fold_out_box<T: Clone, P: Policy + Clone + Conjunction<()>>
                    (bbox_vec : Vec<BBox<T, P>>) -> Result<BBox<Vec<T>, P>, ()> {
    let values = bbox_vec
                        .clone().into_iter()
                        .map(|bbox| bbox.clone().temporary_unbox().clone())
                        .collect();
    let policies_vec: Vec<P> = bbox_vec
                        .clone().into_iter()
                        .map(|bbox| bbox.clone().policy().clone())
                        .collect();
    //if policies_vec.len() > 0 {
        let base = policies_vec[0].clone(); 
        let composed_policy = policies_vec
                            .into_iter()
                            .fold(base,  //base 0th instead of reduce bc don't need to unwrap()
                                |acc, elem|
                                acc.join(&elem).unwrap());
        Ok(BBox::new(values, composed_policy))
    //} else {
        //not sure of desired behavior - useless BBox for len 0 vector? would need same type P
        //Ok(BBox::new(values, NoPolicy{ })) 
        //Err("Folding box out of empty vector - no policies to fold")
    //    Err(())
    //}
}


pub fn fold_in_box<T: Clone, P: Policy + Clone + Conjunction<()>>
                    (boxed_vec : BBox<Vec<T>, P>) -> Vec<BBox<T, P>> {
    let policy = boxed_vec.clone().policy().clone(); 
    boxed_vec.clone().temporary_unbox().clone()
            .into_iter()
            .map(|item: T| BBox::new(item, policy.clone()))
            .collect()
}


