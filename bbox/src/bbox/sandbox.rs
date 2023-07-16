use crate::bbox::BBox;

// BBox and containers of it are sandboxable.
pub trait Sandboxable<T> {
    type Out;
    fn smap<F: Fn(&BBox<T>) -> T>(&self, lambda: F) -> Self::Out;
}
impl<T> Sandboxable<T> for BBox<T> {
    type Out = T;
    fn smap<F: Fn(&BBox<T>) -> T>(&self, lambda: F) -> T {
        lambda(self)
    }
}
impl<T, S: Sandboxable<T>> Sandboxable<T> for Vec<S> {
    type Out = Vec<S::Out>;
    fn smap<F: Fn(&BBox<T>) -> T>(&self, lambda: F) -> Self::Out {
        self.iter().map(|e| e.smap(&lambda)).collect()
    }
}
impl<T, S: Sandboxable<T>> Sandboxable<T> for &Vec<S> {
    type Out = Vec<S::Out>;
    fn smap<F: Fn(&BBox<T>) -> T>(&self, lambda: F) -> Self::Out {
        self.iter().map(|e| e.smap(&lambda)).collect()
    }
}

// Sandbox execute with a container of bboxes.
pub fn sandbox_execute<T: Clone, S: Sandboxable<T>, R, F: FnOnce(S::Out) -> R>(
    s: S,
    lambda: F,
) -> BBox<R> {
    let v = s.smap(|b| b.t.clone());
    // TODO(artem): merge policies together
    BBox::new(lambda(v), vec![])
}

// Need to generalize this to many arguments.
pub fn sandbox_combine<
    T1: Clone,
    T2: Clone,
    S1: Sandboxable<T1>,
    S2: Sandboxable<T2>,
    R,
    F: FnOnce(S1::Out, S2::Out) -> R,
>(
    s1: S1,
    s2: S2,
    lambda: F,
) -> BBox<R> {
    let v1 = s1.smap(|b| b.t.clone());
    let v2 = s2.smap(|b| b.t.clone());
    // TODO(artem): merge policies together
    BBox::new(lambda(v1, v2), vec![])
}
