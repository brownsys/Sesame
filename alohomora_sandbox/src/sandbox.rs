use std::time::Instant;
#[cfg(not(target_arch = "wasm32"))]
use crate::SandboxInstance;
use crate::SandboxableType;
use crate::SandboxOut;

/// Trait that sandboxed functions should implement.
/// Do not implement directly, instead decorate the sandbox fn with #[crate::fast_transfer::AlohomoraSandbox()].
pub trait AlohomoraSandbox<T: SandboxableType, R: SandboxableType> {
    /// The actual sandbox function.
    #[cfg(target_arch = "wasm32")]
    fn function(arg: T) -> R;

    /// The FFI function responsible for invoking this sandbox.
    #[cfg(not(target_arch = "wasm32"))]
    fn ffi(arg: *mut std::ffi::c_void, sandbox: usize) -> *mut std::ffi::c_void;

    /// Overall Entry point from application land.
    /// Invokes the sandbox on the given argument and returns the result.
    /// Responsible for:
    /// 1. Acquiring a lock on some available sandbox in the pool.
    /// 2. Transferring arg to the sandbox memory, either using FastSandboxTransfer or bincode serialization.
    /// 3. Invoking the sandbox via FFI.
    /// 4. Transferring the returned result out of the sandbox memory, either using FastSandboxTransfer or bincode
    ///    deserialization.
    /// 5. Cleaning up the sandbox for future reuse.
    #[cfg(not(target_arch = "wasm32"))]
    fn sandbox_entrypoint(arg: T) -> SandboxOut<R> {
        // Lock a sandbox and create an allocator for it.
        #[cfg(feature = "sandbox_timing")]
        let timer = Instant::now();
        let sandbox = SandboxInstance::new();
        #[cfg(feature = "sandbox_timing")]
        let timing_setup = timer.elapsed();

        // fast_transfer the arg into the sandbox, including any required unswizzling.
        #[cfg(feature = "sandbox_timing")]
        let timer = Instant::now();
        let arg_ptr: *mut std::ffi::c_void = SandboxableType::into_sandbox(arg, sandbox);
        #[cfg(feature = "sandbox_timing")]
        let timing_serialize = timer.elapsed();

        // Call the sandbox FFI function passing it the arg located in the sandbox.
        #[cfg(feature = "sandbox_timing")]
        let timer = Instant::now();
        let return_ptr = Self::ffi(arg_ptr, sandbox.index());
        #[cfg(feature = "sandbox_timing")]
        let timing_ffi = timer.elapsed();

        // Transfer the return value back to the application, including any required swizzling.
        #[cfg(feature = "sandbox_timing")]
        let timer = Instant::now();
        let result: SandboxOut<R> = SandboxableType::out_of_sandbox(return_ptr, sandbox);
        #[cfg(feature = "sandbox_timing")]
        let timing_deserialize = timer.elapsed();

        // Release sandbox.
        #[cfg(feature = "sandbox_timing")]
        let timer = Instant::now();
        sandbox.release();
        #[cfg(feature = "sandbox_timing")]
        let timing_teardown = timer.elapsed();

        // Return result.
        #[cfg(feature = "sandbox_timing")]
        return SandboxOut {
            total: Default::default(),
            function: result.function,
            setup: timing_setup,
            teardown: timing_teardown,
            serialize: timing_serialize + result.serialize,
            deserialize: timing_deserialize + result.deserialize,
            ffi: timing_ffi,
            fold: Default::default(),
            ret: result.ret,
        };

        #[cfg(not(feature = "sandbox_timing"))]
        return result;
    }

    /// Entry point within the sandbox.
    /// This is executed *within the sandbox* (with a 32 arch).
    /// Pointers here are 32 bits, and require no further swizzling or unswizzling.
    /// Responsible for:
    /// 1. Casting/deserializing the argument pointer to the final argument type.
    /// 2. Calling the sandboxed function with that type.
    /// 3. Casting/serializing the return value.
    #[cfg(target_arch = "wasm32")]
    fn sandbox_preamble(arg: *mut std::ffi::c_void) -> *mut std::ffi::c_void {
        // Reconstruct actual data from ffi pointer
        #[cfg(feature = "sandbox_timing")]
        let timer = Instant::now();
        let arg: T = SandboxableType::data_from_ptr(arg);
        #[cfg(feature = "sandbox_timing")]
        let timing_serialize = timer.elapsed();

        // Call the actual function
        #[cfg(feature = "sandbox_timing")]
        let timer = Instant::now();
        let ret: R = Self::function(arg);
        #[cfg(feature = "sandbox_timing")]
        let timing_function = timer.elapsed();

        // Convert output into pointer for passing back through ffi
        #[cfg(feature = "sandbox_timing")]
        return SandboxableType::ptr_from_data(SandboxOut {
            total: Default::default(),
            function: timing_function,
            setup: Default::default(),
            teardown: Default::default(),
            serialize: timing_serialize,
            deserialize: Default::default(),
            ffi: Default::default(),
            fold: Default::default(),
            ret: ret,
        });

        #[cfg(not(feature = "sandbox_timing"))]
        return SandboxableType::ptr_from_data(ret);
    }
}
