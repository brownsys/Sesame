use crate::bbox::BBox;

// Move BBox inside and outside a vec.
impl<T> From<BBox<Vec<T>>> for Vec<BBox<T>> {
    fn from(x: BBox<Vec<T>>) -> Vec<BBox<T>> {
        // TODO(artem): think about how this interacts with the policies
        let p = x.p;
        x.t.into_iter().map(|t| BBox::new(t, p.clone())).collect()
    }
}
impl<T> From<Vec<BBox<T>>> for BBox<Vec<T>> {
    fn from(x: Vec<BBox<T>>) -> BBox<Vec<T>> {
        // TODO(artem): think about how this interacts with the policies
        BBox::new(x.into_iter().map(|b| b.t).collect(), vec![])
    }
}
