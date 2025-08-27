use crate::bbox::BBox;
use crate::policy::Policy;

// An extension is essentially a specific closure we allow to consume the internals of a BBox and
// return an arbitrary data type (not necessarily protected).
//
// The reasoning behind only offering these two APIs is that the other option suddenly becomes to
// have 6 methods :
// - Owned data and self
// - Owned data, borrowed immut self
// - Owned data, borrowed mutable self
// - Borrowed data, owned self
// - Borrowed data, borrowed immut self
// - Borrowed data, borrowed mutable self
//
// Here instead, extensions can be defined over both base types and their references.
// Because extensions are to be used sparingly and are expected to be written by the Tahini team
// (or at least reviewed by them), we consider this an okay effort.
pub trait SesamePConExtension<T, P: Policy, R>
where Self: Sized {
    fn apply(self, data: T, policy: P) -> R;
    fn apply_ref(self, data: &T, policy: &P) -> R;
}

impl<T, P: Policy> BBox<T, P> {
    pub fn apply_extension<R, E: SesamePConExtension<T, P, R>>(self, extension: E) -> R {
        let (t, p) = self.consume();
        extension.apply(t, p)
    }
    pub fn apply_extension_ref<R, E: SesamePConExtension<T, P, R>>(&self, extension: E) -> R {
        let t = self.data();
        let p = self.policy();
        extension.apply_ref(t, p)
    }
}