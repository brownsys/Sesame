/*
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

// Run scrutinizer
fn run_scrutinizer(env: &Env) {
    warn!("\x1b[96mnote: \x1b[97mRunning scrutinizer....\x1b[0m");

    let mut cmd = Command::new(&env.cargo);
    remove_vars(&mut cmd);

    let status = cmd
        .arg("+nightly-2023-08-25")
        .arg("scrutinizer")
        .args(["--config-path", "scrutinizer-config.toml"])
        .env("RUST_BACKTRACE", "full")
        .env("RUST_LOG", "scrutinizer=trace,scrutils=trace")
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
        run_scrutinizer(env);
        warn!("\x1b[92m    Scrutinizer completed!\x1b[0m");
    } else {
        warn!("\x1b[96mnote: \x1b[97min {} mode without Scrutinizer\x1b[0m", profile);
    }
}
*/
