use rustc_lint::LateContext;
use rustc_span::def_id::DefId;

use clippy_utils::get_trait_def_id;
use clippy_utils::diagnostics::span_lint_and_help;

use std::vec::Vec;

declare_alohomora_lint!(
    /// ### What it does
    /// Denies manual implementations of AlohomoraType

    /// ### Why is this bad?
    /// Developers must derive impls of AlohomoraType to ensure integrity of data protection.

    /// ### Example
    /// ```rust
    /// // impl AlohomoraType for BadStruct { ... }
    /// ```
    /// Use instead:
    /// ```rust
    /// // #[derive(AlohomoraType)]
    /// // #[out_type(name = "GoodStructOut", to_derive = [Debug])]
    /// // pub struct GoodStruct { ... }    /// ```
    pub ALOHOMORA_TYPE,
    Deny, // does not allow override
    "AlohomoraType must always be derived, not user-implemented",
    check_crate(cx: &LateContext<'_>)
);

// Check if def_id has the doc attribute we use to detect derived implementations.
fn contains_secret(cx: &LateContext<'_>, def_id: &DefId) -> bool {
    let secret = "Library implementation of AlohomoraType. Do not copy this docstring!";
    cx.tcx.get_attr(*def_id, rustc_span::symbol::Symbol::intern("doc"))
        .and_then(|attr| Some(attr.doc_str().unwrap().to_ident_string()))
        .and_then(|doc| Some(doc.contains(secret)))
        .unwrap_or(false)
}

// Display an error message for offending def_id.
fn error_message(cx: &LateContext<'_>, &def_id: &DefId) {
    let map: rustc_middle::hir::map::Map = cx.tcx.hir();
    match map.span_if_local(def_id.clone()) {
        None => {
            panic!("Manual implementation of AlohomoraType trait at {}. doc: {:?}",
                   cx.tcx.def_path_str(def_id),
                   cx.tcx.get_attr(def_id, rustc_span::symbol::Symbol::intern("doc")));
        },
        Some(span) => {
            span_lint_and_help (
                cx,
                ALOHOMORA_TYPE,
                span,
                "\x1b[93mmanual implementation of AlohomoraType trait\x1b[0m",
                None, "use `#[derive(AlohomoraType)]` instead"
            );
        }
    }
}

// Lint implementation
fn check_crate(cx: &LateContext<'_>) {
    let path: &[&str] = &vec!["alohomora", "AlohomoraType"];
    let aloh_ty_def_id = get_trait_def_id(cx, path);
    if aloh_ty_def_id.is_none() {
        // Compiling some dependency that does not link with Alohomora.
        return;
    }

    let nested_trait_impls = cx.tcx.trait_impls_of(aloh_ty_def_id.unwrap());
    let trait_impls = nested_trait_impls.non_blanket_impls().iter().fold(Vec::new(), |mut acc, (_, v)| { acc.extend(v.iter()); acc });
    trait_impls.iter().filter(|def_id| !contains_secret(cx, def_id)).for_each(|def_id| {
        error_message(cx, def_id);
    });
}


#[test]
fn alohomora_type_legal() {
    dylint_testing::ui_test_example(
        env!("CARGO_PKG_NAME"),
        "alohomora_type_legal"
    );
}

#[test]
fn alohomora_type_illegal() {
    dylint_testing::ui_test_example(
        env!("CARGO_PKG_NAME"),
        "alohomora_type_illegal"
    );
}
