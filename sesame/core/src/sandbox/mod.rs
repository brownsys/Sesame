use std::any::Any;
#[cfg(feature = "sandbox_timing")]
use std::time::Instant;

use crate::bbox::BBox;
use crate::fold::fold;
use crate::policy::{AnyPolicy, PolicyDyn};
use crate::SesameType;

// Expose sesame_sandbox API that controls the interface outside sandbox.
pub use sesame_sandbox::{
    SesameSandbox, FastTransfer, IdentityFastTransfer, SandboxOut, SandboxableType, SandboxInstance
};

#[cfg(feature = "derive")]
pub use sesame_derive::{SesameSandbox, FastTransfer};

/// Copies `t` into a sandbox and executes the specified function on it,
/// and copies the result value and returns it.
pub fn execute_sandbox<S, T, R, PDyn>(t: T) -> SandboxOut<BBox<R, AnyPolicy<PDyn>>>
where
    PDyn: PolicyDyn + ?Sized,
    T: SesameType<dyn Any, PDyn>,
    T::Out: SandboxableType,
    R: SandboxableType,
    S: SesameSandbox<T::Out, R>,
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
