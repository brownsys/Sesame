use std::{fmt::Debug, result};

use crate::AlohomoraType;
use crate::bbox::BBox;
use crate::fold::fold;
use crate::policy::AnyPolicy;

// Expose alohomora_sandbox API that controls the interface outside sandbox.
pub use alohomora_sandbox::{AlohomoraSandbox, SandboxableType, FastSandboxTransfer};

#[cfg(feature = "alohomora_derive")]
pub use alohomora_derive::AlohomoraSandbox;

/// Copies `t` into a sandbox and executes the specified function on it,
/// and copies the result value and returns it.
pub fn execute_sandbox<S, T, R>(t: T) -> BBox<R, AnyPolicy>
where
    T: AlohomoraType,
    T::Out: SandboxableType,
    R: SandboxableType,
    S: AlohomoraSandbox<T::Out, R>,
{
    // Remove boxes from args.
    let outer_boxed = fold::<AnyPolicy, _, _>(t).unwrap();
    let (t, p) = outer_boxed.consume();

    // Invoke sandbox.
    let result = S::sandbox_entrypoint(t);

    BBox::new(result, p)
}
