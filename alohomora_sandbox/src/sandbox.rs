#[cfg(not(target_arch = "wasm32"))]
use crate::SandboxInstance;
use crate::SandboxableType;

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
    fn sandbox_entrypoint(arg: T) -> R {
        // Lock a sandbox and create an allocator for it.
        let sandbox = SandboxInstance::new();

        // fast_transfer the arg into the sandbox, including any required unswizzling.
        let arg_ptr: *mut std::ffi::c_void = SandboxableType::into_sandbox(arg, sandbox);

        // Call the sandbox FFI function passing it the arg located in the sandbox.
        let return_ptr = Self::ffi(arg_ptr, sandbox.index());

        // Transfer the return value back to the application, including any required swizzling.
        let result = R::out_of_sandbox(return_ptr, sandbox);

        // Release sandbox.
        sandbox.release();

        // Return result.
        result
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
        let arg: T = SandboxableType::data_from_ptr(arg);

        // Call the actual function
        let ret: R = Self::function(arg);

        // Convert output into pointer for passing back through ffi
        SandboxableType::ptr_from_data(ret)
    }
}