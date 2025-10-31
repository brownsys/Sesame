use std::fs::metadata;
use std::io::Write;

use serde::Serialize;

use crate::SesameBuilder;

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

#[derive(Serialize, Clone, Debug)]
pub struct RLBoxConfiguration {
    pub root: String,
    pub include: Vec<String>,
    pub wasm2c: String,
    pub wasi_clang: String,
    pub wasi_sysroot: String,
    pub wasi_runtime_files: Vec<String>,
    pub library_path: String,
}

pub fn fetch_and_build_rlbox_wasm2c_sandbox(builder: &SesameBuilder) -> RLBoxConfiguration {
    let rlbox_wasm2c_sandbox_path =
        format!("{}/{}", builder.env.out_directory, "rlbox_wasm2c_sandbox");
    builder.logger.info(
        "rlbox",
        &format!(
            "rlbox_wasm2_sandbox directory: {}",
            rlbox_wasm2c_sandbox_path
        ),
    );

    // Check if rlbox_wasm2c_sandbox was already cloned.
    match metadata(rlbox_wasm2c_sandbox_path.clone()) {
        Ok(metadata) => {
            if metadata.is_dir() {
                builder
                    .logger
                    .warn("RLBox", "rlbox directory exists, pulling");
                // Already cloned, pull.
                let status = builder
                    .command("RLBox Pull", "git")
                    .current_dir(&rlbox_wasm2c_sandbox_path)
                    .arg("pull")
                    .execute()
                    .expect(
                        "Failed to pull 'https://github.com/AllenAby/rlbox_wasm2c_sandbox.git'",
                    );
                if !status.success() {
                    builder.logger.error(
                        "rlbox",
                        "Failed to pull 'https://github.com/AllenAby/rlbox_wasm2c_sandbox.git'",
                    );
                    panic!("");
                }
            } else {
                builder.logger.error(
                    "rlbox",
                    &format!(
                        "rlbox_wasm2c_sandbox directory is a file {}",
                        rlbox_wasm2c_sandbox_path
                    ),
                );
                panic!("");
            }
        }
        Err(_) => {
            builder.logger.warn("RLBox", "Cloning RLBox");
            // Clone.
            let status = builder
                .command("RLBox Clone", "git")
                .arg("clone")
                .arg("https://github.com/AllenAby/rlbox_wasm2c_sandbox.git")
                .arg(&rlbox_wasm2c_sandbox_path)
                .execute()
                .expect("Failed to clone 'https://github.com/AllenAby/rlbox_wasm2c_sandbox.git'");
            if !status.success() {
                builder.logger.error(
                    "rlbox",
                    "Failed to clone 'https://github.com/AllenAby/rlbox_wasm2c_sandbox.git'",
                );
                panic!("");
            }

            // Configure CMake (Once).
            builder.logger.warn("RLBox", "Configuring cmake");
            let status = builder
                .command("Configure cmake", "cmake")
                .current_dir(&rlbox_wasm2c_sandbox_path)
                .args(["-S", ".", "-B", "./build", "-DCMAKE_BUILD_TYPE=Release"])
                .execute()
                .expect("Failed to configure cmake for rlbox_was2mc_sandbox");
            if !status.success() {
                builder.logger.error(
                    "rlbox",
                    "Failed to configure cmake for rlbox_was2mc_sandbox.",
                );
                panic!("");
            }
        }
    }

    // Repo is cloned, and cmake is configured.
    // Build with cmake.
    builder.logger.warn("RLBox", "Building RLBox with cmake");
    let status = builder
        .command("Build RLBox with cmake", "cmake")
        .current_dir(&rlbox_wasm2c_sandbox_path)
        .args(["--build", "./build", "--target", "all"])
        .execute()
        .expect("Failed to build with cmake for rlbox_was2mc_sandbox");
    if !status.success() {
        builder.logger.error(
            "rlbox",
            "Failed to build with cmake for rlbox_was2mc_sandbox",
        );
        panic!("");
    }

    let output = RLBoxConfiguration {
        root: rlbox_wasm2c_sandbox_path.clone(),
        include: INCLUDE_DIRS
            .iter()
            .map(|p| format!("{}/{}", rlbox_wasm2c_sandbox_path, p))
            .collect(),
        wasm2c: format!("{}/{}", rlbox_wasm2c_sandbox_path, WASM2C),
        wasi_clang: format!("{}/{}", rlbox_wasm2c_sandbox_path, WASI_CLANG),
        wasi_sysroot: format!("{}/{}", rlbox_wasm2c_sandbox_path, WASI_SYSROOT),
        wasi_runtime_files: RT_FILES
            .iter()
            .map(|p| format!("{}/{}", rlbox_wasm2c_sandbox_path, p))
            .collect(),
        library_path: format!("{}/{}", rlbox_wasm2c_sandbox_path, LIBRARY_PATH),
    };

    builder
        .logger
        .info("RLBox", &format!("RLBox configuration {:#?}", output));
    output
}
