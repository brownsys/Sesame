use crate::policy::{NoPolicy, Policy};
// pub type PconExtensionClosure<T, P, R> = fn(T, P) -> R;


pub trait SesamePConExtension<T, P: Policy, R> {
    // type ExtensionClosure: Fn(T, P) -> R;
    // fn apply_once(self, data: T, policy: P) -> R;
    fn apply_mut(&mut self, data: T, policy: P) -> R {
        todo!()
    }
    fn apply(&self, data: T, policy: P) -> R {
        todo!()
    }
    fn apply_ref(&self, data: &T, policy: &P) -> R {
        todo!()
    }
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
