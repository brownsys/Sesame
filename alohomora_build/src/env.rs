use std::path::Path;
use std::env::var;
use std::process::Command;

use cargo_toml::Manifest;

use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct Env {
    // Name of crate currently being compiled.
    pub current_crate_name: String,
    // Path to the top level cargo wrapper/command.
    pub cargo: String,
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
    // Host triplet (e.g. linux_x86_64_unknown)
    pub host: String,
}

impl Env {
    pub fn lib_name(&self) -> String {
        let package = self.cargo_toml.package.as_ref().unwrap();
        package.name.clone()
    }
    pub fn file_exists(&self, file: &str) -> bool {
        Path::new(&format!("{}/{}", self.package_directory, file))
            .try_exists()
            .unwrap_or(false)
    }
}

// Find the default target triplet (i.e. the host architecture).
fn find_host() -> String {
    let output = Command::new("rustc")
        .arg("-vV")
        .output()
        .expect("Cannot get default rustc target");
    let stdout = String::from_utf8(output.stdout)
        .expect("Cannot get default rustc target stdout");

    for part in stdout.split("\n") {
        if part.starts_with("host: ") {
            let host: String = part.chars().skip("host: ".len()).collect();
            if host.len() > 0 {
                return host;
            }
        }
    }

    panic!("Cannot find host");
}

fn find_cargo() -> String {
    for (k, v) in std::env::vars() {
        println!("{}: {}", k, v);
    }
    let (_, cargo) = std::env::vars()
        .filter(|(key, _)| key == "_")
        .last()
        .unwrap_or((String::from("_"), String::from("cargo")));
    cargo
}

pub fn read_env() -> Env {
    let current_crate_name = var("CARGO_PKG_NAME").unwrap();
    let cargo = find_cargo();
    let package_directory = var("CARGO_MANIFEST_DIR").unwrap();
    let out_directory = var("OUT_DIR").unwrap();
    let target = var("TARGET").unwrap();
    let working_directory = var("PWD").unwrap();
    let profile = var("PROFILE").unwrap();
    let cargo_toml = Manifest::from_path(&format!("{}/{}", package_directory, "Cargo.toml")).unwrap();
    let host = find_host();
    Env {
        current_crate_name,
        cargo,
        cargo_toml,
        package_directory,
        out_directory,
        target,
        working_directory,
        profile,
        host,
    }
}