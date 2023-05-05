pub struct BBox<T: Clone> {
    pub t: T,

}

impl<T: Clone> BBox<T> {
    pub fn unbox(&self, ctx: &str) -> &T {
        &self.t
    }

    pub fn new(t: T) -> Self {
        Self { t }
    }

    pub fn into2<F: Clone>(&self) -> BBox<F> where T: Into<F> {
        BBox::new(self.t.clone().into())
    }

    pub fn internal_unbox(&self) -> &T {
        &self.t
    }

    pub fn sandbox_execute<R: Clone, F: Fn(&T) -> R>(&self, lambda: F) -> BBox<R> {
        BBox::new(lambda(&self.t))
    }

    pub fn sandbox_combine<U: Clone, V: Clone, R: Clone, F: Fn(&U, &V) -> R>(bbox1: BBox<U>, bbox2: BBox<V>, lambda: F) -> BBox<R> {
        BBox::new(lambda(bbox1.internal_unbox(), bbox2.internal_unbox()))
    }
}