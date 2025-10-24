use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::Write;

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
}
impl Logger {
    pub fn new(log_file: &str) -> Result<Self, std::io::Error> {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_file)?;

        Ok(Self {
            file: RefCell::new(file),
        })
    }

    fn log(&self, cargo_level: &str, color: &str, component: &str, level: &str, msg: &str) {
        let msg = format!("Sesame Build: {}: {}: {}\n", component, level, msg);
        self.file.borrow_mut().write_all(msg.as_bytes()).unwrap();
        self.file.borrow_mut().flush().unwrap();
        if cargo_level != "" {
            println!(
                "cargo:{}=\r{}{}{}{}{}{}{}",
                cargo_level, ESCAPE, color, END, msg, ESCAPE, RESET, END
            );
        }
    }

    pub fn warn(&self, component: &str, msg: &str) {
        self.log("warning", YELLOWFG, component, "Warning", msg);
    }
    pub fn error(&self, component: &str, msg: &str) {
        self.log("error", REDFG, component, "Error", msg);
    }
    pub fn info(&self, component: &str, msg: &str) {
        self.log("", BLUEFG, component, "Info", msg);
    }
    pub fn success(&self, component: &str, msg: &str) {
        self.log("warning", GREENFG, component, "Success", msg);
    }

    pub fn log_bash_environment(&self) {
        let mut kvmap = BTreeMap::new();
        for (k, v) in std::env::vars() {
            kvmap.insert(k, v);
        }
        let mut string = String::new();
        for (k, v) in kvmap {
            string += &format!("{}={}\n", k, v);
        }
        self.info("Bash Environment", &string);
    }
}
