use std::process::{Command};

use crate::env::Env;

// Generates wrappers.
pub fn build_library_wasm(env: &Env) -> String {
    warn!("\x1b[96mnote: \x1b[97mBuilding WASM package....\x1b[0m");

    let output = Command::new(&env.cargo)
        .arg("+nightly-2023-10-06")
        .arg("build")
        .arg("--release")
        .arg("--lib")
        .args(["-Z", "build-std=std,panic_abort"])
        .args(["--target", "wasm32-rlbox.json"])
        .args(["--target-dir", &format!("{}/{}", env.package_directory, "wasm_target")])
        .arg("--verbose")
        .env_remove("RUSTFLAGS")
        .env_remove("RUSTCFLAGS")
        .env_remove("RUST_LOG")
        .output()
        .expect("\x1b[91merror: \x1b[97mFailed to build sandboxes library with wasm'.\x1b[0m");
    if !output.status.success() {
        eprintln!("-----===============================================-------");
        eprintln!("{}", String::from_utf8(output.stdout).unwrap());
        eprintln!("{}", String::from_utf8(output.stderr).unwrap());
        eprintln!("-----===============================================-------");
        panic!("\x1b[91merror: \x1b[97mFailed to build sandboxes library with wasm'.\x1b[0m");
    }

    format!("{}/{}/{}", env.working_directory, "wasm_target/wasm32-rlbox/release", env.lib_name())
}
