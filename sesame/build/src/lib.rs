use crate::command::Command;#[allow(dead_code)]
use crate::env::Env;
use crate::error::Error;
use crate::logging::Logger;

mod command;
mod dylints;
mod env;
mod error;
mod logging;
mod sandbox;
mod scrutinizer;

pub struct SesameBuilder {
    env: Env,
    logger: Logger,
}
impl SesameBuilder {
    pub fn new(log_file: &str) -> Result<SesameBuilder, Error> {
        let env = env::read_env()?;
        let logger = Logger::new(log_file)?;
        logger.info("Environment", &format!("{:#?}", env));
        logger.log_bash_environment();
        Ok(SesameBuilder { env, logger })
    }

    /// If build.rs invokes this function, then Sesame will run scrutinizer during release builds.
    pub fn scrutinizer(&mut self) {
        if self.env.target != "wasm32-rlbox" {
            // scrutinizer::scrutinize(&self.env);
        }
    }

    /// Build any sandboxes within this lib.
    pub fn build_sandbox(&mut self) {
        self.logger.warn("Sandbox Build", &format!("Entry point with target {}", self.env.target));
        if self.env.target != "wasm32-rlbox" {
            self.logger.warn("Sandbox Build", "target is not wasm32-rlbox");
            sandbox::build_sandbox(self);
        } else {
            self.logger.success("Sandbox Build", "target is wasm32-rlbox! Skipping!");
        }
    }

    /// Link against sandbox libs.
    /// Call this function repeatedly to link against several sandbox libs.
    pub fn link_sandbox(&mut self, sandbox_dir: &str) {
        if self.env.target != "wasm32-rlbox" {
            println!(
                "cargo:rustc-link-search=native={}/{}",
                self.env.package_directory, sandbox_dir
            );
            println!(
                "cargo:rustc-link-arg=-Wl,-rpath,{}/{}",
                self.env.package_directory, sandbox_dir
            );
        }
    }

    /// If build.rs invokes this function, then Sesame will run the lints during release builds.
    pub fn lints(&mut self) {
        if self.env.target != "wasm32-rlbox" {
            //dylints::run_lints(&self.env);
        }
    }

    /// Create a command for exeuction.
    pub(crate) fn command(&self, title: &str, program: &str) -> Command {
        Command::new(&self.logger, title, program)
    }
}

/*
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
*/
