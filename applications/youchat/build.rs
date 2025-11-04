use sesame_build::{Options, SesameBuilder};

fn main() {
    let _ = SesameBuilder::new(Options::new().verbose(false));
}
