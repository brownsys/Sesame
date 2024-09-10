use std::fmt::{Debug, Formatter};
use std::time::{Duration, Instant};
use crate::{SandboxInstance, SandboxableType};

#[cfg(not(target_arch = "wasm32"))]
use crate::pointers::SandboxPtr;

// Timing information.
pub struct SandboxTimingInfo<R> {
    pub total: Duration,
    pub function: Duration,
    pub setup: Duration,
    pub teardown: Duration,
    pub serialize: Duration,
    pub deserialize: Duration,
    pub ffi: Duration,
    pub fold: Duration,
    pub ret: R,
}

// Can print timing information.
impl<R> Debug for SandboxTimingInfo<R> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "total: {:?}, function: {:?}, setup: {:?}, teardown: {:?},\
             serialize: {:?}, deserialzie: {:?}, ffi: {:?}, fold: {:?}",
            self.total, self.function, self.setup, self.teardown,
            self.serialize, self.deserialize, self.ffi, self.fold,
        ))
    }
}

// Temporary struct that stores a 32bit pointer to ret.
struct SandboxedSandboxTimingInfo {
    pub total: Duration,
    pub function: Duration,
    pub setup: Duration,
    pub teardown: Duration,
    pub serialize: Duration,
    pub deserialize: Duration,
    pub ffi: Duration,
    pub fold: Duration,
    pub ret: u32,
}

// Implement `FastTransfer` for boxes of `FastTransfer` types.
#[doc = "Library implementation of FastSandboxTransfer. Do not copy this docstring!"]
impl<R: SandboxableType> SandboxableType for SandboxTimingInfo<R> {
    #[cfg(not(target_arch = "wasm32"))]
    fn into_sandbox(_: Self, _: SandboxInstance) -> *mut std::ffi::c_void {
        panic!("Unreachable code!");
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn out_of_sandbox(ptr: *mut std::ffi::c_void, sandbox: SandboxInstance) -> Self {
        let ptr = ptr as *mut SandboxedSandboxTimingInfo;
        let ptr = unsafe { Box::from_raw_in(ptr, sandbox) };

        let data_ptr = SandboxPtr::new(ptr.ret).swizzle(sandbox).ptr();
        let data = R::out_of_sandbox(data_ptr, sandbox);

        SandboxTimingInfo {
            total: ptr.total,
            function: ptr.function,
            setup: ptr.setup,
            teardown: ptr.teardown,
            serialize: ptr.serialize,
            deserialize: ptr.deserialize,
            ffi: ptr.ffi,
            fold: ptr.fold,
            ret: data,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn data_from_ptr(ptr: *mut std::ffi::c_void) -> Self {
        panic!("Unreachable code!");
    }

    #[cfg(target_arch = "wasm32")]
    fn ptr_from_data(data: Self) -> *mut std::ffi::c_void {
        let timer = Instant::now();
        let mut data = SandboxedSandboxTimingInfo {
            total: data.total,
            function: data.function,
            setup: data.setup,
            teardown: data.teardown,
            serialize: data.serialize,
            deserialize: data.deserialize,
            ffi: data.ffi,
            fold: data.fold,
            ret: R::ptr_from_data(data.ret) as u32,
        };
        data.deserialize = data.deserialize + timer.elapsed();

        // Put value into box
        Box::into_raw(Box::new(data)) as *mut std::ffi::c_void
    }
}
