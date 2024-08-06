extern crate alohomora;
use alohomora_sandbox::Sandboxable;

#[derive(Debug)]
pub struct Example {
    pub a: u32,
    pub b: u64,
    pub s: String,
}

impl Sandboxable for Example {
    type InSandboxUnswizzled = Example;
    fn into_sandbox(outside: Self, _: alohomora_sandbox::alloc::SandboxAllocator) -> Self::InSandboxUnswizzled {
        println!("{:?}", outside);
        todo!()
    }

    fn out_of_sandbox(inside: &Self::InSandboxUnswizzled, _: usize) -> Self {
        println!("{:?}", inside);
        todo!()
    }
}

pub fn main() {}
