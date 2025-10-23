use serde::Serialize;

use crate::env::Env;
use crate::sandbox::metadata::get_sandboxes;
use crate::sandbox::template::template;
use crate::sandbox::rlbox::RLBoxConfiguration;

// Render context for rendering the wrappers template.
#[derive(Serialize)]
pub struct RenderContext {
    pub name: String,            // Library name (must match whats in Cargo.toml).
    pub sandboxes: Vec<String>,  // Name of every sandbox entry function.
    pub env: Env,
    pub rlbox: RLBoxConfiguration,
}

// Fill Render context given environment.
impl From<(&Env, &RLBoxConfiguration)> for RenderContext {
    fn from((env, rlbox): (&Env, &RLBoxConfiguration)) -> Self {
        RenderContext {
            name: env.lib_name(),
            sandboxes: get_sandboxes(&env.cargo_toml),
            env: env.clone(),
            rlbox: rlbox.clone(),
        }
    }
}

// The rendered wrappers.
pub struct Wrappers {
    pub makefile: String,
    pub wrapper_cpp: String,
    pub wrapper_h: String,
    pub wasi_rt_aux_c: String,
    pub wasm32_rlbox_json: String,
}

pub fn render(env: &Env, rlbox: &RLBoxConfiguration) -> Wrappers {
    // Construct TinyTemplate instance.
    let tt = template();

    // Fill in rendering context based on environment.
    let context = RenderContext::from((env, rlbox));

    // Render the templates.
    Wrappers {
        makefile: tt.render("Makefile", &context).unwrap(),
        wrapper_cpp: tt.render("wrapper_cpp", &context).unwrap(),
        wrapper_h: tt.render("wrapper_h", &context).unwrap(),
        wasi_rt_aux_c: tt.render("wasi_rt_aux_c", &context).unwrap(),
        wasm32_rlbox_json: tt.render("wasm32_rlbox_json", &context).unwrap(),
    }
}
