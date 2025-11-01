use sesame_build::{Options, SesameBuilder};

fn main() {
    let builder = SesameBuilder::new(Options::new().verbose(false));
    builder.unwrap().build_sandbox();
}

