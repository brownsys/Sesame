// Macro for printing warnings to developers.
macro_rules! warn {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[32;1mAlohomora build:{}", format!($($tokens)*))
    }
}

mod env;
mod dylints;
mod sandbox;
mod scrutinizer;

// Applications and libraries should call this from their build.rs
pub fn alohomora_build(scrutinize: bool, sandbox_directories: &[&str]) {
    let env = env::read_env();

    warn!("\x1b[96mBuilding {} in {}....\x1b[0m", env.current_crate_name, env.package_directory);

    if env.target != "wasm32-rlbox" {
        sandbox::build_sandbox(&env);
        // dylints::run_lints(&env);
        if scrutinize {
            scrutinizer::scrutinize(&env);
        }
    }


    for dir in sandbox_directories {
        println!("cargo:rustc-link-search=native={}/{}", env.package_directory, dir);
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}/{}", env.package_directory, dir);
    }
    
    warn!("\x1b[92mFinished building {}\x1b[0m", env.current_crate_name);
}
