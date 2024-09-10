use std::{fmt::Debug};

use std::time::Instant;

use crate::AlohomoraType;
use crate::bbox::BBox;
use crate::fold::fold;
use crate::policy::AnyPolicy;

// Expose alohomora_sandbox API that controls the interface outside sandbox.
pub use alohomora_sandbox::{AlohomoraSandbox, SandboxableType, FastTransfer, IdentityFastTransfer, SandboxOut};

#[cfg(feature = "alohomora_derive")]
pub use alohomora_derive::{AlohomoraSandbox, FastTransfer};

/// Copies `t` into a sandbox and executes the specified function on it,
/// and copies the result value and returns it.
pub fn execute_sandbox<S, T, R>(t: T) -> SandboxOut<BBox<R, AnyPolicy>>
where
    T: AlohomoraType,
    T::Out: SandboxableType,
    R: SandboxableType,
    S: AlohomoraSandbox<T::Out, R>,
{
    #[cfg(feature = "sandbox_timing")]
    let timer = Instant::now();

    // Remove boxes from args.
    let outer_boxed = fold(t).unwrap();
    let (t, p) = outer_boxed.consume();

    #[cfg(feature = "sandbox_timing")]
    let timing_fold = timer.elapsed();

    // Invoke sandbox.
    let result = S::sandbox_entrypoint(t);

    // Return result with or without timing depending on feature.
    #[cfg(feature = "sandbox_timing")]
    {
        let mut result = SandboxOut {
            total: Default::default(),
            function: result.function,
            setup: result.setup,
            teardown: result.teardown,
            serialize: result.serialize,
            deserialize: result.deserialize,
            ffi: result.ffi,
            fold: timing_fold,
            ret: BBox::new(result.ret, p),
        };
        result.total = timer.elapsed();
        return result;
    }

    #[cfg(not(feature = "sandbox_timing"))]
    return BBox::new(result, p);
}
