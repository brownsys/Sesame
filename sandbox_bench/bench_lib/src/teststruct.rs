#[derive(Debug)]
pub struct TestStructUnswizzled {
    my_int: u32,       // 4
    my_float: f32,     // 4
    my_float2: f64,    // 8 <- 16 total
    ptr_to_buddy: u32, // 8b
}

#[derive(Debug)]
pub struct TestStructReal {
    _unswizzled: *mut TestStructUnswizzled,
    my_int: u32,
    my_float: f32,
    my_float2: f64,
    ptr_to_buddy: *mut i32, // maybe this should be wrapped in another thing to make sure we can call
}

// pub unsafe fn swizzle(unswizzled: *mut TestStructUnswizzled) -> TestStructReal {
//     TestStructReal {
//         _unswizzled: unswizzled,
//         my_int: (*unswizzled).my_int,
//         my_float: (*unswizzled).my_float,
//         my_float2: (*unswizzled).my_float2,
//         ptr_to_buddy: ::alohomora_sandbox::ptr::swizzle_ptr(
//             (*unswizzled).ptr_to_buddy,
//             unswizzled as *mut std::ffi::c_void,
//         ),
//     }
// }

// impl TestStructReal {
//     pub unsafe fn unswizzle(&self) {
//         (*self._unswizzled).my_int = self.my_int;
//         (*self._unswizzled).my_float = self.my_float;
//         (*self._unswizzled).my_float2 = self.my_float2;
//         (*self._unswizzled).ptr_to_buddy = ::alohomora_sandbox::ptr::unswizzle_ptr(self.ptr_to_buddy);
//     }
// }
