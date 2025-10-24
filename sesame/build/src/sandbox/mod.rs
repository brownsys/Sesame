use crate::SesameBuilder;

mod metadata;
mod render;
mod rlbox;
mod template;
mod wasm;
mod wrappers;

pub fn build_sandbox(builder: &SesameBuilder) {
    let sandboxes = metadata::get_sandboxes(&builder.env.cargo_toml);
    if sandboxes.len() > 0 {
        // We need to build the sandbox into a .so.
        let output_so = format!("lib{}_sandbox.so", builder.env.lib_name());

        builder.logger.info(
            "Sandbox Build",
            &format!(
                "Building {} sandboxed functions to `{}`",
                sandboxes.len(),
                output_so
            ),
        );
        if !builder.env.file_exists(&output_so) {
            builder.logger.warn(
                "Sandbox Build",
                &format!(
                    "Building sandbox from scratch because `{}` does not exist",
                    output_so
                ),
            );
            // Fetch rlbox_wasm2c_sandbox from github and build it.
            let rlbox = rlbox::fetch_and_build_rlbox_wasm2c_sandbox(builder);

            // Generate wrapper files by rendering stubs under sandbox_stubs.
            wrappers::generate_wrappers(builder, &rlbox);

            // Build library using wasm32-rlbox toolchain.
            wasm::build_library_wasm(builder);

            // Build wrappers and link with wasi runtime and decompiled wasm module.
            wrappers::build_wrappers(builder);
        } else {
            builder.logger.warn(
                "Sandbox Build",
                &format!("Skip building sandbox because `{}` exists", output_so),
            );
        }

        // Link with produced so.
        println!(
            "cargo:rustc-link-search=native={}",
            builder.env.package_directory
        );
        println!(
            "cargo:rustc-link-lib=dylib={}_sandbox",
            builder.env.lib_name()
        );
        println!(
            "cargo:rustc-link-arg=-Wl,-rpath,{}",
            builder.env.package_directory
        );

        builder.logger.success(
            "Sandbox Build",
            &format!("Sandbox built successfully at `{}`", output_so),
        );
    }
}
