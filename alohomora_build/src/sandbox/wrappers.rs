use std::fs;
use std::process::Command;

use crate::env::Env;
use crate::sandbox::render::render;
use crate::sandbox::rlbox::RLBoxConfiguration;

// Generates wrappers.
pub fn generate_wrappers(env: &Env, rlbox: &RLBoxConfiguration) {
    warn!("\x1b[96mnote: \x1b[97mGenerating wrappers in {}....\x1b[0m", env.out_directory);

    // Render the templates given the environment.
    let wrappers = render(env, rlbox);

    // Write to files.
    fs::write(&format!("{}/{}", env.package_directory, "wasm32-rlbox.json"), wrappers.wasm32_rlbox_json).unwrap();
    fs::write(&format!("{}/{}", env.out_directory, "Makefile"), wrappers.makefile).unwrap();
    fs::write(&format!("{}/{}", env.out_directory, "wrapper.cpp"), wrappers.wrapper_cpp).unwrap();
    fs::write(&format!("{}/{}", env.out_directory, "wrapper.h"), wrappers.wrapper_h).unwrap();
    fs::write(&format!("{}/{}", env.out_directory, "wasi_rt.aux.c"), wrappers.wasi_rt_aux_c).unwrap();
}

pub fn build_wrappers(env: &Env) {
    warn!("\x1b[96mnote: \x1b[97mBuilding wrappers....\x1b[0m");

    let status = Command::new("make")
        .current_dir(&env.out_directory)
        .status()
        .expect("\x1b[91merror: \x1b[97mFailed to build wrappers.\x1b[0m");
    if !status.success() {
        panic!("\x1b[91merror: \x1b[97mFailed to build wrappers.\x1b[0m");
    }
}