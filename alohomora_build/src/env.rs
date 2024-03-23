use std::env::var;

use cargo_toml::Manifest;

use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Env {
    // Parsed Cargo.toml of the crate being compiled.
    pub cargo_toml: Manifest,
    // Directory of where Cargo.toml of the crate being compiled is.
    pub package_directory: String,
    // Working directory for the build under <package>/target/... where we should place files.
    pub out_directory: String,
    // Target architecture, e.g. wasm32-rlbox, x86_64-unknown-linux-gnu, ...
    pub target: String,
    // Working directory of build script.
    pub working_directory: String,
    // Profile of build, e.g. release, debug, etc
    pub profile: String,
}

impl Env {
    pub fn lib_name(&self) -> String {
        let package = self.cargo_toml.package.as_ref().unwrap();
        package.name.clone()
    }
}

pub fn read_env() -> Env {
    let package_directory = var("CARGO_MANIFEST_DIR").unwrap();
    let out_directory = var("OUT_DIR").unwrap();
    let target = var("TARGET").unwrap();
    let working_directory = var("PWD").unwrap();
    let profile = var("PROFILE").unwrap();
    let cargo_toml = Manifest::from_path(&format!("{}/{}", package_directory, "Cargo.toml")).unwrap();
    Env {
        cargo_toml,
        package_directory,
        out_directory,
        target,
        working_directory,
        profile,
    }
}