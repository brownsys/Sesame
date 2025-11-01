use crate::logging::Logger;
use serde::Serialize;
use std::ffi::OsStr;
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use tinytemplate::TinyTemplate;

pub struct CommandOutput {
    pub status: ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

pub struct CommandResult<'a> {
    title: String,
    result: Result<CommandOutput, std::io::Error>,
    logger: &'a Logger,
}
impl<'a> CommandResult<'a> {
    pub fn ok(title: String, output: CommandOutput, logger: &'a Logger) -> CommandResult<'a> {
        CommandResult {
            title,
            result: Ok(output),
            logger,
        }
    }
    pub fn err(title: String, err: std::io::Error, logger: &'a Logger) -> CommandResult<'a> {
        CommandResult {
            title,
            result: Err(err),
            logger,
        }
    }
    pub fn expect(self, error: &str) -> CommandOutput {
        match self.result {
            Ok(t) => t,
            Err(err) => {
                let msg = format!("{}: {:?}", error, err);
                self.logger.error(&self.title, &msg);
                panic!("{}", msg);
            }
        }
    }
}

pub struct Command<'a> {
    title: String,
    command: std::process::Command,
    env_cleared: bool,
    logger: &'a Logger,
}

// Mimic Command API.
impl<'a> Command<'a> {
    pub(crate) fn new<S: AsRef<OsStr>>(logger: &'a Logger, title: &str, cmd: S) -> Self {
        Self {
            logger,
            title: title.to_owned(),
            command: std::process::Command::new(cmd),
            env_cleared: false,
        }
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
    #[allow(dead_code)]
    pub fn env<K, V>(&mut self, key: K, val: V) -> &mut Self
    where
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.env(key, val);
        self
    }
    #[allow(dead_code)]
    pub fn envs<I, K, V>(&mut self, vars: I) -> &mut Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<OsStr>,
        V: AsRef<OsStr>,
    {
        self.command.envs(vars);
        self
    }
    #[allow(dead_code)]
    pub fn env_remove<K: AsRef<OsStr>>(&mut self, key: K) -> &mut Self {
        self.command.env_remove(key);
        self
    }
    #[allow(dead_code)]
    pub fn env_clear(&mut self) -> &mut Self {
        self.env_cleared = true;
        self.command.env_clear();
        self
    }

    // Executes the command redirecting its stdout and stderr to the log file.
    pub fn execute(&mut self) -> CommandResult<'a> {
        self.log();

        let result = self
            .command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match result {
            Ok(output) => {
                let status = output.status;
                let stdout = String::from_utf8(output.stdout).unwrap();
                let stderr = String::from_utf8(output.stderr).unwrap();
                self.logger.info(&self.title, &format!("stdout ---------------------\n{}\n----------------------------------------------------\n", stdout));
                self.logger.info(&self.title, &format!("stderr ---------------------\n{}\n----------------------------------------------------\n", stderr));
                CommandResult::ok(
                    self.title.clone(),
                    CommandOutput {
                        status,
                        stdout,
                        stderr,
                    },
                    self.logger,
                )
            }
            Err(err) => {
                self.logger
                    .error(&self.title, &format!("Failed to execute: {}", err));
                CommandResult::err(self.title.clone(), err, self.logger)
            }
        }
    }
}

// For logging: internal.
// Spawning logs the command.
impl<'a> Command<'a> {
    fn log(&self) {
        let mut tt = TinyTemplate::new();
        tt.add_template("command", include_str!("command.tt"))
            .unwrap();

        let context = self.logging_context();
        let msg = tt.render("command", &context).unwrap();

        self.logger.info(&self.title, &msg);
    }
    fn logging_context(&self) -> CommandContext {
        let mut env = Vec::new();
        if self.env_cleared {
            env.push(EnvContext {
                key: String::from("+CLEANED+"),
                value: String::from("+CLEANED+"),
            });
        }
        for (k, v) in self.command.get_envs() {
            let k = k.to_str().unwrap_or("+NOT_ASCII+").to_owned();
            let v = v
                .map(|v| v.to_str().unwrap_or("+NOT_ASCII+"))
                .unwrap_or("+CLEANED+")
                .to_owned();
            env.push(EnvContext { key: k, value: v });
        }
        CommandContext {
            title: self.title.clone(),
            env,
            workdir: self
                .command
                .get_current_dir()
                .map(|d| d.to_str().unwrap_or("+EMPTY+"))
                .unwrap_or("+EMPTY+")
                .to_owned(),
            command: self
                .command
                .get_program()
                .to_str()
                .unwrap_or("+NOT_ASCII+")
                .to_owned(),
            args: self
                .command
                .get_args()
                .map(|arg| arg.to_str().unwrap_or("+EMPTY+").to_owned())
                .collect(),
        }
    }
}

#[derive(Serialize)]
struct EnvContext {
    pub key: String,
    pub value: String,
}

#[derive(Serialize)]
struct CommandContext {
    pub title: String,
    pub env: Vec<EnvContext>,
    pub workdir: String,
    pub command: String,
    pub args: Vec<String>,
}
