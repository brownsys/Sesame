use std::cell::RefCell;
use std::ffi::OsStr;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::{Child, ExitStatus, Output};

// ANSI colors and escape symbols.
const ESCAPE: &'static str = "\x1b[";
const RESET: &'static str = "0";
const END: &'static str = "m";
const REDFG: &'static str = "31";
const REDBG: &'static str = "41";
const GREENFG: &'static str = "32";
const GREENBG: &'static str = "42";
const YELLOWFG: &'static str = "93";
const YELLOWBG: &'static str = "103";
const BLUEFG: &'static str = "34";
const BLUEBG: &'static str = "44";

pub struct Logger {
    file: RefCell<File>,
}
impl Logger {
    pub fn new(log_file: &str) -> Result<Self, std::io::Error> {
        let mut file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(log_file)?;

        Ok(Self { file: RefCell::new(file) })
    }

    fn log(&self, cargo_level: &str, color: &str, component: &str, level: &str, msg: &str) {
        let msg = format!("Sesame Build: {}: {}: {}", component, level, msg);
        self.file.borrow_mut().write_all(msg.as_bytes()).unwrap();
        self.file.borrow_mut().flush().unwrap();
        if cargo_level != "" {
            println!("cargo:{}=\r{}{}{}{}{}{}{}", cargo_level, ESCAPE, color, END, msg, ESCAPE, RESET, END);
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
}

pub struct Command {
    title: String,
    command: std::process::Command,
    env_cleaned: bool,
}
impl Command {
    // Mimic Command API.
    pub fn new<S: AsRef<OsStr>>(title: &str, cmd: S) -> Self {
        Self { title: title.to_owned(), command: std::process::Command::new(cmd) }
    }
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Self {
        self.command.arg(arg);
        self
    }
    pub fn args<I, S>(&mut self, args: I) -> &mut Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.command.args(args);
        self
    }
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Self {
        self.command.current_dir(dir);
        self
    }
    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.env(key, val);
        self
    }
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.envs(vars);
        self
    }
    pub fn env_remove<K: AsRef<OsStr>>(&mut self, key: K) -> &mut Self {
        self.command.env_remove(key);
        self
    }
    pub fn env_clear(&mut self) -> &mut Self {
        self.command.env_clear();
        self
    }

    // Spawning logs the command.
    // TODO(babman): log commands, replace used commands in code to use the new command struct
    //               log outputs and errors based on verbosity.
    pub fn log(logger: &Logger) {

        logger.info();
    }

    pub fn output(&mut self) -> Result<Output, std::io::Error> {
        self.command.output()
    }
    pub fn status(&mut self) -> Result<ExitStatus, std::io::Error> {
        self.command.status()
    }
}