use sesame_build::SesameBuilder;

fn main() {
    let builder = SesameBuilder::new("/tmp/log.log");
    builder.unwrap().build_sandbox();
}

