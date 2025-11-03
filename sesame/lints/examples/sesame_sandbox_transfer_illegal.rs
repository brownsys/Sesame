extern crate alohomora;
use alohomora_sandbox::FastTransfer;

#[derive(Debug)]
pub struct Example {
    pub a: u32,
    pub b: u64,
    pub s: String,
}

impl FastTransfer for Example {
    type TypeInSandbox = Example;
    fn into_sandbox(outside: Self, _: alohomora_sandbox::SandboxInstance) -> Self::TypeInSandbox {
        println!("{:?}", outside);
        todo!()
    }

    fn out_of_sandbox(inside: &Self::TypeInSandbox, _: alohomora_sandbox::SandboxInstance) -> Self {
        println!("{:?}", inside);
        todo!()
    }
}

pub fn main() {}
