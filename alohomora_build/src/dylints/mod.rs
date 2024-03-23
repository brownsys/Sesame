use std::process::Command;

use crate::env::Env;

pub fn run_lints(env: &Env) {
    let profile = &env.profile;
    if profile == "release" {
        warn!("\x1b[97m    Running Alohomora lints...\x1b[0m");
        let lint_res = Command::new("cargo")
            .arg("dylint")
            // TODO(babman): Update dylint to a more recent version to be able to use `--git <repo> --pattern alohomora_lints/`
            .arg("--lib")
            .arg("alohomora_lints")
            .status()
            .expect("cargo dylint failed");

        if !lint_res.success() {
            panic!("\x1b[91merror: \x1b[97malohomora lints failed! See above for manual implementations to replace.\x1b[0m");
        } else {
            warn!("\x1b[92m    Alohomora lints passed!\x1b[0m");
        }
    } else {
        warn!("\x1b[96mnote: \x1b[97min {} mode without Alohomora lints\x1b[0m", profile);
    }
}
