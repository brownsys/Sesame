#![feature(rustc_private)]
#![warn(unused_extern_crates)]
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

// Helper for declaring lints without too much boilerplate.
macro_rules! declare_alohomora_lint {
    ($(#[$attr:meta])* $vis:vis $NAME:ident, $Level:ident, $desc:expr, $func:ident($($param:ident: $ty:ty),*)) => {
        paste::paste! {
            rustc_session::declare_lint!($(#[$attr])* $vis $NAME, $Level, $desc);
            rustc_session::declare_lint_pass!([<$NAME:camel>] => [$NAME]);

            // Trait implementation.
            impl<'tcx> rustc_lint::LateLintPass<'tcx> for [<$NAME:camel>] {
                fn $func(&mut self, $($param: $ty),*) {
                    $func($($param,)*);
                }
            }

            // Register lint.
            impl [<$NAME:camel>] {
                pub fn register(lint_store: &mut rustc_lint::LintStore) {
                    lint_store.register_lints(&[$NAME]);
                    lint_store.register_late_pass(|_| Box::new([<$NAME:camel>]));
                }
            }
        }
    };
}

// Declare that we are a dylint library.
dylint_linting::dylint_library!();

// List all lints, make each lint its own mod.
mod alohomora_pcr; 
mod alohomora_sandbox;
mod alohomora_type;
mod alohomora_sandbox_transfer;
mod alohomora_sandbox_identity_transfer;

// Register all lints.
#[allow(clippy::no_mangle_with_rust_abi)]
#[no_mangle]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    dylint_linting::init_config(sess);
    alohomora_pcr::AlohomoraPcr::register(lint_store); 
    alohomora_sandbox::AlohomoraSandbox::register(lint_store);
    alohomora_type::AlohomoraType::register(lint_store);
    alohomora_sandbox_transfer::AlohomoraSandboxTransfer::register(lint_store);
    alohomora_sandbox_identity_transfer::AlohomoraSandboxIdentityTransfer::register(lint_store);
}
