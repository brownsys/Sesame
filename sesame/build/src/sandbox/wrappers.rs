use std::fs;

use crate::sandbox::render::render;
use crate::sandbox::rlbox::RLBoxConfiguration;
use crate::SesameBuilder;

// Generates wrappers.
pub fn generate_wrappers(builder: &SesameBuilder, rlbox: &RLBoxConfiguration) {
    builder.logger.warn(
        "Wrappers",
        &format!("Generating wrappers in {}", builder.env.out_directory),
    );

    // Render the templates given the environment.
    let wrappers = render(&builder.env, rlbox);

    // Write to files.
    fs::write(
        &format!("{}/{}", builder.env.package_directory, "wasm32-rlbox.json"),
        wrappers.wasm32_rlbox_json,
    )
    .unwrap();
    fs::write(
        &format!("{}/{}", builder.env.out_directory, "Makefile"),
        wrappers.makefile,
    )
    .unwrap();
    fs::write(
        &format!("{}/{}", builder.env.out_directory, "wrapper.cpp"),
        wrappers.wrapper_cpp,
    )
    .unwrap();
    fs::write(
        &format!("{}/{}", builder.env.out_directory, "wrapper.h"),
        wrappers.wrapper_h,
    )
    .unwrap();
    fs::write(
        &format!("{}/{}", builder.env.out_directory, "wasi_rt.aux.c"),
        wrappers.wasi_rt_aux_c,
    )
    .unwrap();
}

pub fn build_wrappers(builder: &SesameBuilder) {
    builder.logger.info("Wrappers", "Building wrappers");

    let status = builder
        .command("Build Wrappers", "make")
        .current_dir(&builder.env.out_directory)
        .execute()
        .expect("Failed to build wrappers");
    if !status.success() {
        builder
            .logger
            .error("Build Wrappers", "Failed to build wrappers");
        panic!("");
    }
}
