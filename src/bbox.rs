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

    pub fn sandbox_unbox<R, F: Fn(&T) -> R>(&self, lambda: F) -> R {
        lambda(&self.t)
    }
}
