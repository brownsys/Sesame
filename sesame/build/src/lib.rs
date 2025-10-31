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

// Options for builder.
pub struct Options {
    pub(self) log_file: Option<String>,
    pub(self) verbose: bool,
}
impl Options {
    pub fn new() -> Self {
        Self {
            log_file: None,
            verbose: true,
        }
    }
    pub fn log_file(&mut self, log_file: &str) -> &mut Self {
        self.log_file = Some(log_file.to_string());
        self
    }
    pub fn verbose(&mut self, verbose: bool) -> &mut Self {
        self.verbose = verbose;
        self
    }
}

pub struct SesameBuilder {
    env: Env,
    logger: Logger,
}
impl SesameBuilder {
    pub fn new(opts: &Options) -> Result<SesameBuilder, Error> {
        let env = env::read_env()?;
        let logger = match &opts.log_file {
            None => Logger::new(&env, opts.verbose)?,
            Some(log_file) => Logger::with_file(log_file, opts.verbose)?,
        };
        if opts.verbose {
            logger.info("Environment", &format!("\n{:#?}\n\n\n", env));
            logger.log_bash_environment();
        }
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
        if self.env.target != "wasm32-rlbox" {
            sandbox::build_sandbox(self);
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
