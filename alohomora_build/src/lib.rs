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
    if env.target != "wasm32-rlbox" {
        dylints::run_lints(&env);
        sandbox::build_sandbox(&env);
    }
}
