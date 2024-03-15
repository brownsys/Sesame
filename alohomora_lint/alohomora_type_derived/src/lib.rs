#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_middle;
extern crate rustc_span;

use rustc_lint::{LateContext, LateLintPass};
use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::get_trait_def_id;

use rustc_span::def_id::DefId; 
use rustc_middle::ty::fast_reject::SimplifiedType;

use std::vec::Vec;


dylint_linting::declare_late_lint! {
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
    pub ALOHOMORA_TY_DERIVED,
    Deny, //does not allow override
    "AlohomoraType must always be derived, not user-implemented"
}

impl<'tcx> LateLintPass<'tcx> for AlohomoraTyDerived {

    fn check_crate(&mut self, cx: &LateContext<'tcx>) {
        let path: &[&str] = &vec!["alohomora", "AlohomoraType"];
        let aloh_ty_did: Option<DefId> = get_trait_def_id(cx, path); 
        if aloh_ty_did.is_none() {
            return; 
        } 
        let aloh_ty_did = aloh_ty_did.unwrap(); 

        let secret = "Library implementation of AlohomoraType. Do not copy this docstring!"; 
        let contains_secret = |def_id: DefId| cx.tcx.get_attr(def_id, rustc_span::symbol::Symbol::intern("doc"))
                                    .and_then(|attr| Some(attr.doc_str().unwrap().to_ident_string()))
                                    .and_then(|doc| Some(doc.contains(secret)))
                                    .unwrap_or(false);  

        let map: rustc_middle::hir::map::Map = cx.tcx.hir(); 
        let error_message = |&def_id: &DefId| {
            let span = map.span_if_local(def_id.clone()); 
            //TODO(corinn) cleaner solution for this: 
            if span.is_none(){
                panic!("Manual implementation of AlohomoraType trait at {}.
                doc: {:?}", cx.tcx.def_path_str(def_id), cx.tcx.get_attr(def_id, rustc_span::symbol::Symbol::intern("doc"))); 
            } else {
                span_lint_and_help (
                    cx,
                    ALOHOMORA_TY_DERIVED,
                    span.unwrap(),
                    "\x1b[93mmanual implementation of AlohomoraType trait\x1b[0m", 
                    None, "use `#[derive(AlohomoraType)]` instead"
                );
            }
        };

        let trait_impls: &rustc_middle::ty::trait_def::TraitImpls = cx.tcx.trait_impls_of(aloh_ty_did);
        let _blanket_bad_impls: Vec<DefId> = 
                                trait_impls.blanket_impls()
                                    .iter()
                                    .filter(|&def_id| !contains_secret(*def_id))
                                    .cloned()
                                    .collect();  

        /* trait_impls.non_blanket_impls() .iter()
                                .for_each(|(_, v)| v.iter()
                                    .for_each(|&def_id| 
                                        println!("implementation of AlohomoraType trait at {}", cx.tcx.def_path_str(def_id))));
        */

        let non_blanket_bad_impls: Vec<(&SimplifiedType, &Vec<DefId>)> =  trait_impls.non_blanket_impls()
                                    .iter()
                                    .filter(|&(_, v)|
                                        v.iter()
                                            .any(|&def_id| !contains_secret(def_id)))
                                    .collect();
        
        //TODO: blanket impls not local -> panics. assume we do need to fail for blanket?
        //blanket_bad_impls.iter()
            //.for_each(|def_id| error_message(def_id)); 
        
                          
        non_blanket_bad_impls.iter()
            .for_each(|&(_, v)| {
                 v.iter()
                    .for_each(|def_id| {    
                        error_message(def_id)
                    });
                }
            );
         
    }

}

#[test]
fn ui() {
    dylint_testing::ui_test(
        env!("CARGO_PKG_NAME"),
        &std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui"),
    );
}

