pub struct BBox<T> {
    pub t: T,

}

impl<T> BBox<T> {
    pub fn unbox(&self, ctx: &str) -> &T {
        &self.t
    }

    pub fn new(t: T) -> Self {
        Self { t }
    }

    pub fn internal_unbox(&self) -> &T {
        &self.t
    }

    pub fn sandbox_execute<R, F: Fn(&T) -> R>(&self, lambda: F) -> BBox<R> {
        BBox::new(lambda(&self.t))
    }

    pub fn sandbox_combine<U, V, R, F: Fn(&U, &V) -> R>(bbox1: BBox<U>, bbox2: BBox<V>, lambda: F) -> BBox<R> {
        BBox::new(lambda(bbox1.internal_unbox(), bbox2.internal_unbox()))
    }
}
