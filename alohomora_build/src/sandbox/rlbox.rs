use std::io::Write;
use std::process::Command;
use std::fs::metadata;

use serde::Serialize;

use crate::env::Env;

const INCLUDE_DIRS: [&str; 3] = [
    "include",
    "build/_deps/rlbox-src/code/include",
    "build/_deps/mod_wasm2c-src/wasm2c",
];
const WASM2C: &str = "build/_deps/mod_wasm2c-src/bin/wasm2c";
const WASI_CLANG: &str = "build/_deps/wasiclang-src/bin/clang";
const WASI_SYSROOT: &str = "build/_deps/wasiclang-src/share/wasi-sysroot";
const LIBRARY_PATH: &str = "build/_deps/wasiclang-src/share/wasi-sysroot/lib/wasm32-wasi";

const RT_FILES: [&str; 4] = [
    "build/_deps/mod_wasm2c-src/wasm2c/wasm-rt-impl.c",
    "build/_deps/mod_wasm2c-src/wasm2c/wasm-rt-exceptions-impl.c",
    "src/wasm2c_rt_mem.c",
    "src/wasm2c_rt_minwasi.c",
];

#[derive(Serialize,Clone)]
pub struct RLBoxConfiguration {
    pub root: String,
    pub include: Vec<String>,
    pub wasm2c: String,
    pub wasi_clang: String,
    pub wasi_sysroot: String,
    pub wasi_runtime_files: Vec<String>,
    pub library_path: String,
}

pub fn fetch_and_build_rlbox_wasm2c_sandbox(env: &Env) -> RLBoxConfiguration {
    let rlbox_wasm2c_sandbox_path = format!("{}/{}", env.out_directory, "rlbox_wasm2c_sandbox");

    // Check if rlbox_wasm2c_sandbox was already cloned.
    warn!("\x1b[96mnote: \x1b[97mrlbox_wasm2c_sandbox directory is '{}'\x1b[0m", rlbox_wasm2c_sandbox_path);
    match metadata(rlbox_wasm2c_sandbox_path.clone()) {
        Ok(metadata) => {
            if metadata.is_dir() {
                warn!("\x1b[96mnote: \x1b[97mPulling rlbox....\x1b[0m");
                // Already cloned, pull.
                let status = Command::new("git")
                    .current_dir(&rlbox_wasm2c_sandbox_path)
                    .arg("pull")
                    .status()
                    .expect("\x1b[91merror: \x1b[97mFailed to pull 'https://github.com/AllenAby/rlbox_wasm2c_sandbox.git'.\x1b[0m");
                if !status.success() {
                    panic!("\x1b[91merror: \x1b[97mFailed to pull 'https://github.com/AllenAby/rlbox_wasm2c_sandbox.git'.\x1b[0m");
                }
            } else {
                panic!("\x1b[91merror: rlbox_wasm2c_sandbox directory is a file {}'.\x1b[0m", rlbox_wasm2c_sandbox_path);
            }
        },
        Err(_) => {
            warn!("\x1b[96mnote: \x1b[97mCloning rlbox....\x1b[0m");
            // Clone.
            let status = Command::new("git")
                .arg("clone")
                .arg("https://github.com/AllenAby/rlbox_wasm2c_sandbox.git")
                .arg(&rlbox_wasm2c_sandbox_path)
                .status()
                .expect("\x1b[91merror: \x1b[97mFailed to fetch 'https://github.com/AllenAby/rlbox_wasm2c_sandbox.git'.\x1b[0m");
            if !status.success() {
                panic!("\x1b[91merror: \x1b[97mFailed to fetch 'https://github.com/AllenAby/rlbox_wasm2c_sandbox.git'.\x1b[0m");
            }

            // Configure CMake (Once).
            let output = Command::new("cmake")
                .current_dir(&rlbox_wasm2c_sandbox_path)
                .args(["-S", ".",  "-B", "./build", "-DCMAKE_BUILD_TYPE=Release"])
                .output()
                .expect("\x1b[91merror: \x1b[97mFailed to configure cmake for rlbox_was2mc_sandbox.\x1b[0m");
            if !output.status.success() {
                std::io::stdout().write_all(&output.stdout).unwrap();
                std::io::stderr().write_all(&output.stderr).unwrap();
                panic!("\x1b[91merror: \x1b[97mFailed to configure cmake for rlbox_was2mc_sandbox.\x1b[0m");
            }
        }
    }

    // Repo is cloned, and cmake is configured.
    // Build with cmake.
    warn!("\x1b[96mnote: \x1b[97mBuilding rlbox....\x1b[0m");
    let output = Command::new("cmake")
        .current_dir(&rlbox_wasm2c_sandbox_path)
        .args(["--build", "./build", "--target", "all"])
        .output()
        .expect("\x1b[91merror: \x1b[97mFailed to build with cmake for rlbox_was2mc_sandbox.\x1b[0m");
    if !output.status.success() {
        std::io::stdout().write_all(&output.stdout).unwrap();
        std::io::stderr().write_all(&output.stderr).unwrap();
        panic!("\x1b[91merror: \x1b[97mFailed to build with cmake for rlbox_was2mc_sandbox.\x1b[0m");
    }

    RLBoxConfiguration {
        root: rlbox_wasm2c_sandbox_path.clone(),
        include: INCLUDE_DIRS.iter()
            .map(|p| format!("{}/{}", rlbox_wasm2c_sandbox_path, p))
            .collect(),
        wasm2c: format!("{}/{}", rlbox_wasm2c_sandbox_path, WASM2C),
        wasi_clang: format!("{}/{}", rlbox_wasm2c_sandbox_path, WASI_CLANG),
        wasi_sysroot: format!("{}/{}", rlbox_wasm2c_sandbox_path, WASI_SYSROOT),
        wasi_runtime_files: RT_FILES.iter()
            .map(|p| format!("{}/{}", rlbox_wasm2c_sandbox_path, p))
            .collect(),
        library_path: format!("{}/{}", rlbox_wasm2c_sandbox_path, LIBRARY_PATH),
    }
}
