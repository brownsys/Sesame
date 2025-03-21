use rustc_lint::LateContext;
use rustc_span::def_id::DefId;

use clippy_utils::get_trait_def_id;
use clippy_utils::diagnostics::span_lint_and_help;

use std::vec::Vec;

declare_alohomora_lint!(
    /// ### What it does
    /// Denies manual implementations of AlohomoraSandbox

    /// ### Why is this bad?
    /// Developers could leak information as it is getting serialized to pass to sandbox

    /// ### Example
    /// ```rust
    /// // impl AlohomoraSandbox for SomeStruct { ... }
    /// ```
    /// Use instead:
    /// ```rust
    /// // #[AlohomoraSandbox()]
    /// // pub fn sandbox_function(arg: Type) -> Type  { ... }    /// ```
    pub ALOHOMORA_SANDBOX_IDENTITY_TRANSFER,
    Deny, // does not allow override
    "IdentityFastTransfer must always be auto-generated by the IdentityFastTransfer macro, not user-implemented",
    check_crate(cx: &LateContext<'_>)
);

// Check if def_id has the doc attribute we use to detect derived implementations.
fn contains_secret(cx: &LateContext<'_>, def_id: &DefId) -> bool {
    let secret = "Library implementation of IdentityFastTransfer. Do not copy this docstring!";
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
            panic!("Manual implementation of IdentityFastTransfer trait at {}. doc: {:?}",
                   cx.tcx.def_path_str(def_id),
                   cx.tcx.get_attr(def_id, rustc_span::symbol::Symbol::intern("doc")));
        },
        Some(span) => {
            span_lint_and_help (
                cx,
                ALOHOMORA_SANDBOX_IDENTITY_TRANSFER,
                span,
                "\x1b[93mmanual implementation of IdentityFastTransfer trait\x1b[0m",
                None, "use `#[derive(IdentityFastTransfer)]` instead"
            );
        }
    }
}

// Lint implementation
fn check_crate(cx: &LateContext<'_>) {
    let path: &[&str] = &vec!["alohomora_sandbox", "IdentityFastTransfer"];
    let def_id = get_trait_def_id(cx, path);
    if def_id.is_none() {
        // Compiling some dependency that does not link with Alohomora.
        return;
    }

    let nested_trait_impls = cx.tcx.trait_impls_of(def_id.unwrap());
    let trait_impls = nested_trait_impls.non_blanket_impls()
        .iter().fold(Vec::new(), |mut acc, (_, v)| { acc.extend(v.iter()); acc });
    
    trait_impls.iter()
        .filter(|def_id| 
            !contains_secret(cx, def_id) 
            && def_id.krate == rustc_hir::def_id::LOCAL_CRATE)
        .for_each(|def_id| {
            error_message(cx, def_id);
        });
}
