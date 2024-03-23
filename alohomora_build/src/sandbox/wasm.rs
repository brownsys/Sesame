use std::process::Command;

use crate::env::Env;

// Generates wrappers.
pub fn build_library_wasm(env: &Env) -> String {
    let status = Command::new("cargo")
        .arg("build")
        .arg("--release")
        .arg("--lib")
        .args(["-Z", "build-std=std,panic_abort"])
        .args(["--target", "wasm32-rlbox.json"])
        .args(["--target-dir", &format!("{}/{}", env.package_directory, "wasm_target")])
        .status()
        .expect("\x1b[91merror: \x1b[97mFailed to build sandboxes library with wasm'.\x1b[0m");
    if !status.success() {
        panic!("\x1b[91merror: \x1b[97mFailed to build sandboxes library with wasm'.\x1b[0m");
    }

    format!("{}/{}/{}", env.working_directory, "wasm_target/wasm32-rlbox/release", env.lib_name())
}