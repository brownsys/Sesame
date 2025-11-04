use crate::ORMPolicy;
use sesame::extensions::{SesameExtension, UncheckedSesameExtension};
use sesame::pcon::PCon;
use std::fmt::{Debug, Formatter};

// sesame_orm's version of a PCon.
#[derive(Clone)]
pub struct ORMPCon<T, P: ORMPolicy> {
    pub(crate) t: T,
    pub(crate) p: P,
}

// ORMPCon is leaky: should improve the below later.
impl<T: Debug, P: ORMPolicy + Debug> Debug for ORMPCon<T, P> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ORMPCon")
            .field("data", &self.t)
            .field("policy", &self.p)
            .finish()
    }
}
impl<T: PartialEq, P: ORMPolicy + PartialEq> PartialEq for ORMPCon<T, P> {
    fn eq(&self, other: &Self) -> bool {
        self.t == other.t && self.p == other.p
    }
}
impl<T: PartialEq + Eq, P: ORMPolicy + PartialEq + Eq> Eq for ORMPCon<T, P> {}

// We use a Sesame extension to transform PCon into an ORMPCon.
struct ORMExtension {}
impl UncheckedSesameExtension for ORMExtension {}
impl<T, P: ORMPolicy> SesameExtension<T, P, ORMPCon<T, P>> for ORMExtension {
    fn apply(&mut self, data: T, policy: P) -> ORMPCon<T, P> {
        ORMPCon { t: data, p: policy }
    }
}

// Conversions from and to ORMPCon.
impl<T, P: ORMPolicy> From<PCon<T, P>> for ORMPCon<T, P> {
    fn from(pcon: PCon<T, P>) -> Self {
        pcon.unchecked_extension(&mut ORMExtension {})
    }
}
impl<T, P: ORMPolicy> Into<PCon<T, P>> for ORMPCon<T, P> {
    fn into(self) -> PCon<T, P> {
        PCon::new(self.t, self.p)
    }
}
