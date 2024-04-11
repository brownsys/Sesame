use std::collections::HashSet;
use std::process::Command;

use crate::env::Env;

fn remove_vars(cmd: &mut Command) {
    let set: HashSet<&'static str> = HashSet::from([
        "PATH",
        "PWD",
        "HOME",
        "HOST",
        // "LD_LIBRARY_PATH",
    ]);
    
    for (key, _) in std::env::vars() {
        if !set.contains(key.as_str()) {
            cmd.env_remove(&key);
        }
    }
}

fn generate_facts(env: &Env) {
    warn!("\x1b[96mnote: \x1b[97mGenerating scrutinizer facts....\x1b[0m");

    let mut cmd = Command::new(&env.cargo);
    remove_vars(&mut cmd);

    let status = cmd
        .arg("+nightly-2023-04-12")
        .arg("build")
        .arg("-Zbuild-std=std,core,alloc,proc_macro")
        .arg(&format!("--target={}", env.host))
        .env("RUSTFLAGS", "-Zalways-encode-mir -Znll-facts")
        .status()
        .expect("\x1b[91merror: \x1b[97mFailed to generate scrutinizer facts'.\x1b[0m");
    if !status.success() {
        panic!("\x1b[91merror: \x1b[97mFailed to generate scrutinizer facts'.\x1b[0m");
    }
}

// Run scrutinizer
fn run_scrutinizer(env: &Env) {
    warn!("\x1b[96mnote: \x1b[97mRunning scrutinizer....\x1b[0m");

    let mut cmd = Command::new(&env.cargo);
    remove_vars(&mut cmd);
    
    let status = cmd
        .arg("+nightly-2023-04-12")
        .arg("scrutinizer")
        .args(["--config-path", "scrutinizer-config.toml"])
        .status()
        .expect("\x1b[91merror: \x1b[97mFailed to run scrutinizer'.\x1b[0m");
    if !status.success() {
        panic!("\x1b[91merror: \x1b[97mFailed to run scrutinizer'.\x1b[0m");
    }
}

pub fn scrutinize(env: &Env) {
    let profile = &env.profile;
    if profile == "release" {
        warn!("\x1b[97m    Scrutinizer\x1b[0m");
        generate_facts(env);
        run_scrutinizer(env);
        warn!("\x1b[92m    Scrutinizer completed!\x1b[0m");
    } else {
        warn!("\x1b[96mnote: \x1b[97min {} mode without Scrutinizer\x1b[0m", profile);
    }
}
