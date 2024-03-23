// Macro for printing warnings to developers.
macro_rules! warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[32;1m{}", format!($($tokens)*))
    }
}

mod env;
mod dylints;
mod sandbox;

// Applications and libraries should call this from their build.rs
pub fn alohomora_build() {
    let env = env::read_env();
    dylints::run_lints(&env);
}

// Sandbox libraries should call this.
pub fn alohomora_sandbox() {
    let env = env::read_env();
    if env.target != "wasm32-rlbox" {
        sandbox::build_sandbox(&env);
    }
}