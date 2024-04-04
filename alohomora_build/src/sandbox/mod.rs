use crate::env::Env;

mod metadata;
mod render;
mod rlbox;
mod template;
mod wasm;
mod wrappers;

pub fn build_sandbox(env: &Env) {
    let sandboxes = metadata::get_sandboxes(&env.cargo_toml);
    if sandboxes.len() > 0 {
        warn!("\x1b[97m    Sandbox\x1b[0m");

        // We need to build the sandbox into a .so.
        let output_so = format!("lib{}_sandbox.so", env.lib_name());
        if !env.file_exists(&output_so) {
            // Fetch rlbox_wasm2c_sandbox from github and build it.
            let rlbox = rlbox::fetch_and_build_rlbox_wasm2c_sandbox(env);

            // Generate wrapper files by rendering stubs under sandbox_stubs.
            wrappers::generate_wrappers(env, &rlbox);

            // Build library using wasm32-rlbox toolchain.
            wasm::build_library_wasm(env);

            // Build wrappers and link with wasi runtime and decompiled wasm module.
            wrappers::build_wrappers(env);
        } else {
            warn!("\x1b[96mnote: \x1b[97mSkip rebuilding sandbox because {} exists....\x1b[0m", output_so);
        }

        // Link with produced so.
        println!("cargo:rustc-link-search=native={}", env.package_directory);
        println!("cargo:rustc-link-lib=dylib={}_sandbox", env.lib_name());
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", env.package_directory);

        warn!("\x1b[92m    Sandbox completed!\x1b[0m");
    }
}
