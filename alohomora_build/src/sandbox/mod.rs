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
      // Fetch rlbox_wasm2c_sandbox from github and build it.
      let rlbox = rlbox::fetch_and_build_rlbox_wasm2c_sandbox(env);
      // Generate wrapper files by rendering stubs under sandbox_stubs.
      wrappers::generate_wrappers(env, &rlbox);
      // Build library using wasm32-rlbox toolchain.
      wasm::build_library_wasm(env);
      // Build wrappers and link with wasi runtime and decompiled wasm module.
      wrappers::build_wrappers(env);

      // Link with produced so.
      println!("cargo:rustc-link-search=native={}", env.out_directory);
      println!("cargo:rustc-link-lib=dylib=myapp_lib_sandbox");
      println!("cargo:rustc-link-arg=-Wl,-rpath={}", env.out_directory);
    }
}
