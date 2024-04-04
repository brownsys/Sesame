use std::process::Command;

use crate::env::Env;

mod metadata;

pub fn run_lints(env: &Env) {
    let profile = &env.profile;
    if profile == "release" {
        warn!("\x1b[97m    Running Alohomora lints...\x1b[0m");
        
        let dylint_libraries = metadata::get_dylinting_libraries(&env.cargo_toml);
        if dylint_libraries.len() > 0 {
          let lint_res = Command::new(&env.cargo)
              .arg("+nightly-2023-10-06")
              .arg("dylint")
              .arg("--lib")
              .arg("alohomora_lints")
              .status()
              .expect("cargo dylint failed");

          if !lint_res.success() {
              panic!("\x1b[91merror: \x1b[97malohomora lints failed! See above for manual implementations to replace.\x1b[0m");
          } else {
              warn!("\x1b[92m    Alohomora lints passed!\x1b[0m");
          }
        }
    } else {
        warn!("\x1b[96mnote: \x1b[97min {} mode without Alohomora lints\x1b[0m", profile);
    }
}
