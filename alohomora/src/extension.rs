use crate::policy::{NoPolicy, Policy};
// pub type PconExtensionClosure<T, P, R> = fn(T, P) -> R;


//The reasoning behind only offering these two APIs is that the other option suddenly becomes to
//have 6 methods :
//- Owned data and self
//- Owned data, borrowed immut self
//- Owned data, borrowed mutable self
//- Borrowed data, owned self
//- Borrowed data, borrowed immut self
//- Borrowed data, borrowed mutable self
//
//Here instead, extensions can be defined over both base types and their references.
//Because extensions are to be used sparingly and are expected to be written by the Tahini team
//(or at least reviewed by them), we consider this an okay effort.
pub trait SesamePConExtension<T, P: Policy, R> 
where Self: Sized {
    fn apply(self, data: T, policy: P) -> R;
    fn apply_ref(self, data: &T, policy: &P) -> R;
}


//An extension is essentially a specific closure we assume can consume the internal of a BBox, and
//return an arbitrary data type (not necessarily protected).
//Which is a bit iffy?
//We need to assume extensions can hold some form of state that is modifyable? Which is super
//sketchy security-wise.
//What is the better option here.
//Do we want closures to be stateless so as to be able to run scrutinizer on them perhaps? (we are
//almost sure these are leaky). 
//For example, for serialization, there should be some state that holds the key engine and the
//serializer, the PCon is simply a consumer of the overall ordeal.
//Aren't we reinventing some other form of closure that is simply trusted? 
//I guess the goal here is to have some type of core trusted transformation for libraries to be
//able to rely upon. 
