use std::process::Command;

macro_rules! p {
    ($($tokens: tt)*) => {
        println!("cargo:warning=\r\x1b[32;1m    {}", format!($($tokens)*))
    }
}

fn main() {
    let profile = std::env::var("PROFILE").unwrap();

    if profile.clone().as_str() == "release" {
        p!("\x1b[97mRunning Alohomora lints...\x1b[0m");
        let lint_res = Command::new("cargo")
            .arg("dylint")
            .arg("--all")
            .status()
            .expect("cargo dylint failed"); 
        
        if !lint_res.success() { // color codes on either side for red output
            panic!("\x1b[91mAlohomora lints failed! See above for manual implementations to remove.\x1b[0m"); 
        } else {
            p!("\x1b[92mAlohomora lints passed!\x1b[0m");
        }
    } else {
        p!("\x1b[96mIn {} mode without Alohomora lints\x1b[0m", profile);
    }
}