use alohomora_build::alohomora_build;

fn main() {
    alohomora_build(false, &[]);
    println!("cargo:rerun-if-changed=src/");
}
