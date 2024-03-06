use crate::AlohomoraType;
use crate::fold::fold;
use crate::policy::AnyPolicy;

// Creation of this must be signed.
#[derive(Clone, Copy)]
pub struct PrivacyCriticalRegion<F> {
    f: F,
}
impl<F> PrivacyCriticalRegion<F> {
    pub const fn new(f: F) -> Self {
        PrivacyCriticalRegion { f }
    }
    pub fn get_functor(self) -> F {
        self.f
    }
}

// Executes a PCR over some boxed type.
pub fn execute_pcr<S: AlohomoraType, C, O, F: FnOnce(S::Out, AnyPolicy, C) -> O>(
        data: S,
        functor: PrivacyCriticalRegion<F>,
        arg: C) -> Result<O, ()> {
    let data = fold(data)?;
    let (t, p) = data.consume();
    let functor = functor.get_functor();
    Ok(functor(t, p, arg))
}

/*
// Example of how to use this with a function definition.
//#[PrivacyCriticalRegion("signature")]
fn example(a: u32, _policy: NoPolicy, b: u64) -> String {
    format!("{}{}", a, b)
}

// This part would be auto-generated by the macro.
#[allow(dead_code)]
const EXAMPLE_INSTANCE: PrivacyCriticalRegion<fn(u32, NoPolicy, u64) -> String> = PrivacyCriticalRegion::new(example);

// Example of invoking the PCR.
#[allow(dead_code)]
fn example_inline_lambda() {
    use crate::bbox::BBox;

    let bbox = BBox::new(10, NoPolicy {});
    let result = bbox.into_pcr(EXAMPLE_INSTANCE, 100);
    println!("{}", result);
}
*/