use tinytemplate::{TinyTemplate, format_unescaped};
use tinytemplate::error::Error as TemplateError;

fn double_underscore_formatter(value: &serde_json::Value, str: &mut String) -> Result<(), TemplateError> {
    if let serde_json::value::Value::String(value) = value {
        str.push_str(&value.replace("_", "__"));
        Ok(())
    } else {
        Err(TemplateError::GenericError { msg: String::from("Library name invalid") })
    }
}

// Generates wrappers.
pub fn template() -> TinyTemplate<'static> {
    // Configure tiny template
    let mut tt = TinyTemplate::new();
    tt.set_default_formatter(&format_unescaped);
    tt.add_formatter("double_underscore_formatter", &double_underscore_formatter);

    // Load files at compile time.
    let makefile = include_str!("../../sandbox_stubs/Makefile");
    let wrapper_cpp = include_str!("../../sandbox_stubs/wrapper.cpp");
    let wrapper_h = include_str!("../../sandbox_stubs/wrapper.h");
    let wasi_rt_aux_c = include_str!("../../sandbox_stubs/wasi_rt.aux.c");
    let was32_rlbox_json = include_str!("../../sandbox_stubs/wasm32-rlbox.json");

    // Use TinyTemplate to populate.
    tt.add_template("Makefile", makefile).unwrap();
    tt.add_template("wrapper_cpp", wrapper_cpp).unwrap();
    tt.add_template("wrapper_h", wrapper_h).unwrap();
    tt.add_template("wasi_rt_aux_c", wasi_rt_aux_c).unwrap();
    tt.add_template("wasm32_rlbox_json", was32_rlbox_json).unwrap();

    tt
}