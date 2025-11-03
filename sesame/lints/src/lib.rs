#![feature(rustc_private)]
#![warn(unused_extern_crates)]
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;
extern crate rustc_hir; 

// Helper for declaring lints without too much boilerplate.
macro_rules! declare_sesame_lint {
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
mod sesame_pcr; 
mod sesame_sandbox;
mod sesame_type;
mod sesame_sandbox_transfer;
mod sesame_sandbox_identity_transfer;

// Register all lints.
#[allow(clippy::no_mangle_with_rust_abi)]
#[no_mangle]
pub fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    dylint_linting::init_config(sess);
    sesame_pcr::SesamePcr::register(lint_store); 
    sesame_sandbox::SesameSandbox::register(lint_store);
    sesame_type::SesameType::register(lint_store);
    sesame_sandbox_transfer::SesameSandboxTransfer::register(lint_store);
    sesame_sandbox_identity_transfer::SesameSandboxIdentityTransfer::register(lint_store);
}
