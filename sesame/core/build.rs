use alohomora_build::alohomora_build;

fn main() {
    // let x = std::env::var("CARGO_BIN_NAME");
    // println!("cargo:warning=\r\x1b[32;1mCore build:\x1b[96mnote: \x1b < {}\x1b[0m", x.unwrap_or(String::from("<empty>")));
    // if std::env::var("CARGO_TEST_BUILD").is_ok() {
    println!("cargo:warning=\r\x1b[32;1mCore build:\x1b[96mnote: Sandbox added to path because we are building in dev build....\x1b[0m");
    alohomora_build(false, &["../../tests/sandbox_test_lib"]);
}
