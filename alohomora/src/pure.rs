use crate::AlohomoraType;
use crate::bbox::BBox;
use crate::fold::fold;
use crate::policy::AnyPolicy;

// Creation of this must be checked for purity.
#[derive(Clone, Copy)]
pub struct PrivacyPureRegion<F> {
    f: F,
}
impl<F> PrivacyPureRegion<F> {
    pub const fn new(f: F) -> Self {
        PrivacyPureRegion { f }
    }
    pub fn get_functor(self) -> F {
        self.f
    }
}

// Executes a PCR over some boxed type.
pub fn execute_pure<S: AlohomoraType, O, F: FnOnce(S::Out) -> O>(data: S,  functor: PrivacyPureRegion<F>) -> Result<BBox<O, AnyPolicy>, ()> {
    let data = fold(data)?;
    let (t, p) = data.consume();
    let functor = functor.get_functor();
    Ok(BBox::new(functor(t), p))
}

/*
// Example of how to use this with a lambda.
#[allow(dead_code)]
fn example_inline_lambda() {
    use crate::policy::NoPolicy;
    let bbox = BBox::new(10, NoPolicy {});
    let bbox = execute_pure(
        bbox,
        // Developer writes something like
        // #[PrivacyPureRegion] |x: i32| format!("{}", x);
        // Which expands to:
        PrivacyPureRegion::new(|x: i32| format!("{}", x))
    );
    println!("{}", bbox.unwrap().data());
}

// Example of how to use this with a function definition
//#[PrivacyPureRegion]
fn example_ppr(a: i32) -> String {
    format!("{}", a)
}

// This part would be auto-generated by the macro.
#[allow(dead_code)]
const EXAMPLE_PPR: PrivacyPureRegion<fn(i32) -> String> = PrivacyPureRegion::new(example_ppr);

// Usage
#[allow(dead_code)]
fn example_function_defition() {
    use crate::policy::NoPolicy;
    let bbox = BBox::new(10, NoPolicy {});
    let bbox = execute_pure(bbox, EXAMPLE_PPR);
    println!("{}", bbox.unwrap().data());
}
*/