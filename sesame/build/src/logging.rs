use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::Write;

use crate::env::Env;

// ANSI colors and escape symbols.
#[allow(dead_code)]
const ESCAPE: &'static str = "\x1b[";
#[allow(dead_code)]
const RESET: &'static str = "0";
#[allow(dead_code)]
const END: &'static str = "m";
#[allow(dead_code)]
const REDFG: &'static str = "31";
#[allow(dead_code)]
const REDBG: &'static str = "41";
#[allow(dead_code)]
const GREENFG: &'static str = "32";
#[allow(dead_code)]
const GREENBG: &'static str = "42";
#[allow(dead_code)]
const YELLOWFG: &'static str = "93";
#[allow(dead_code)]
const YELLOWBG: &'static str = "103";
#[allow(dead_code)]
const BLUEFG: &'static str = "34";
#[allow(dead_code)]
const BLUEBG: &'static str = "44";

pub struct Logger {
    file: RefCell<File>,
    verbose: bool,
}
impl Logger {
    pub fn new(env: &Env, verbose: bool) -> Result<Self, std::io::Error> {
        let path = env.log_file_path();
        let file = OpenOptions::new().append(true).create(true).open(&path)?;

        let logger = Self {
            file: RefCell::new(file),
            verbose,
        };
        logger.warn("Logger", &format!("Log file at {}", path));
        Ok(logger)
    }
    pub fn with_file(log_file: &str, verbose: bool) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_file)?;

        let logger = Self {
            file: RefCell::new(file),
            verbose,
        };
        logger.warn("Logger", &format!("Log file at {}", log_file));
        Ok(logger)
    }

    fn log(&self, cargo_level: &str, color: &str, component: &str, level: &str, msg: &str) {
        // Print to file.
        let msg = format!("Sesame Build: {}: {}: {}", component, level, msg);
        self.file.borrow_mut().write_all(msg.as_bytes()).unwrap();
        self.file.borrow_mut().flush().unwrap();

        // Log to
        if cargo_level != "" {
            for line in msg.split('\n') {
                println!(
                    "cargo:{}={}{}{}{}{}{}{}",
                    cargo_level, ESCAPE, color, END, line, ESCAPE, RESET, END
                );
            }
        }
    }

    pub fn warn(&self, component: &str, msg: &str) {
        self.log("warning", YELLOWFG, component, "Warning", msg);
    }
    pub fn error(&self, component: &str, msg: &str) {
        self.log("error", REDFG, component, "Error", msg);
    }
    pub fn info(&self, component: &str, msg: &str) {
        let level = if self.verbose { "warning" } else { "" };
        self.log(level, BLUEFG, component, "Info", msg);
    }
    pub fn success(&self, component: &str, msg: &str) {
        self.log("warning", GREENFG, component, "Success", msg);
    }

    pub fn log_bash_environment(&self) {
        let mut kvmap = BTreeMap::new();
        for (k, v) in std::env::vars() {
            kvmap.insert(k, v);
        }
        let mut string = String::from("\n");
        for (k, v) in kvmap {
            string += &format!("  {} = {}\n", k, v);
        }
        string += "\n\n\n";
        self.info("Bash Environment", &string);
    }
}
