use crate::command::Command;
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
    pub(self) allow_sandbox_printing: bool,
}
impl Options {
    pub fn new() -> Self {
        Self {
            log_file: None,
            verbose: true,
            allow_sandbox_printing: false,
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
    pub fn allow_sandbox_printing(&mut self, allow_sandbox_printing: bool) -> &mut Self {
        self.allow_sandbox_printing = allow_sandbox_printing;
        self
    }
}

pub struct SesameBuilder {
    env: Env,
    logger: Logger,
    allow_sandbox_printing: bool,
}
impl SesameBuilder {
    pub fn new(opts: &Options) -> Result<SesameBuilder, Error> {
        let env = env::read_env()?;
        let logger = match &opts.log_file {
            None => Logger::new(&env, opts.verbose)?,
            Some(log_file) => Logger::with_file(log_file, opts.verbose)?,
        };
        logger.info("Environment", &format!("\n{:#?}\n\n\n", env));
        logger.log_bash_environment();

        let allow_sandbox_printing = opts.allow_sandbox_printing;
        Ok(SesameBuilder {
            env,
            logger,
            allow_sandbox_printing,
        })
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

    /// Create a command for execution.
    pub(crate) fn command(&self, title: &str, program: &str) -> Command {
        Command::new(&self.logger, title, program)
    }

    pub(crate) fn allow_sandbox_printing(&self) -> bool {
        self.allow_sandbox_printing
    }
}
