use sesame_build::{Options, SesameBuilder};

fn main() {
    let builder = SesameBuilder::new(Options::new().verbose(true).allow_sandbox_printing(true));
    builder.unwrap().build_sandbox();
}
