use serde::{Serialize, Deserialize};

use crate::AlohomoraType;
use crate::bbox::BBox;
use crate::fold::fold;
use crate::policy::AnyPolicy;

// Expose alohomora_sandbox API that controls the interface outside sandbox.
pub use alohomora_sandbox::{AlohomoraSandbox, FinalSandboxOut};

#[cfg(feature = "alohomora_derive")]
pub use alohomora_derive::AlohomoraSandbox;


// Main function for executing sandboxes over BBoxed data.
pub fn execute_sandbox<'a, 'b, S, T, R>(t: T) -> BBox<::alohomora_sandbox::FinalSandboxOut<R>, AnyPolicy>
    where
        T: AlohomoraType,
        T::Out: Serialize + Deserialize<'a>,
        R: Serialize + Deserialize<'b>,
        S: AlohomoraSandbox<'a, 'b, T::Out, R>,
{
    let outer_boxed = fold(t).unwrap();
    let (t, p) = outer_boxed.consume();
    BBox::new(S::invoke(t), p)
}
