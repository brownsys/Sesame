#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::sym;

use rustc_ast::ast::LitKind;
use rustc_span::symbol::Symbol;

use rustc_hir::def::Res;
use rustc_hir::Expr;
use rustc_hir::ExprKind;

use rustc_middle::ty::{subst::InternalSubsts, Instance, ParamEnv, TyCtxt};

use sha2::{Sha256, Digest};
use base64::{engine::general_purpose, Engine as _};
use if_chain::if_chain;

use std::fs;
use std::path::Path;
use std::process::Command;
use std::collections::HashSet;
use syn::{self, parse_str};
use quote::ToTokens; 

use scrutils::{Collector, FunctionInfo, 
    compute_deps_for_body, compute_dep_strings_for_crates}; 

declare_alohomora_lint! {
    /// ### What it does
    /// Warns if a CriticalRegion has an invalid signature. 
    /// 
    /// ### Why is this bad?
    /// Closures in Critical Regions must be signed 
    /// to indicate they have been reviewed for unintended leakage 
    /// and for correct use of the application Context. 
    /// 
    /// An invalidated signature indicates that 
    /// the closure, 
    /// a local-crate function the closure calls, 
    /// or a dependency of the closure
    /// has changed.
    /// 
    /// ### Known problems
    /// Signatures permit small changes to closures, 
    /// e.g., adding whitespace or comments, 
    /// but will be invalidated by larger changes 
    /// that are semantically equivalent. 
    /// 
    /// ### Example
    /// ```rust
    /// //  let pcr = PrivacyCriticalRegion::new(|x: u8| { <privacy-critical closure> },
    /// //          Signature {username: "corinnt", 
    /// //              signature: "LS0tLS....."},     // review signature on closure
    /// ```
    pub ALOHOMORA_PCR,
    Warn,
    "highlights and hashes privacy-critical regions", 
    check_expr(cx: &rustc_lint::LateContext<'tcx>, expr: &'_ rustc_hir::Expr<'_>)
}

fn check_expr<'tcx>(cx: &rustc_lint::LateContext<'tcx>, expr: &'_ rustc_hir::Expr<'_>) {
    let target_fn_path: Vec<Symbol> = vec![
        sym!(alohomora), //TODO swap for `sesame`
        sym!(pcr),
        sym!(PrivacyCriticalRegion),
        sym!(new),
    ];

    if let ExprKind::Call(maybe_path, args) = &expr.kind {
        if is_fn_call(cx, maybe_path, target_fn_path) {
            assert!(args.len() == 2); // 2 args to constructor of PrivacyCriticalRegion
            if let ExprKind::Closure(closure) = args[0].kind {
                let closure_body = cx.tcx.hir().body(closure.body);
                let pcr_src = cx
                    .tcx
                    .sess
                    .source_map()
                    .span_to_snippet(closure_body.value.span)
                    .unwrap();
                let correct_src_hash= get_pcr_hash(cx.tcx, closure);
                
                //This arg to CriticalRegion::new will be of type Signature
                let (fn_reviewer_id, fn_reviewer_src_signature ) = extract_from_signature_struct(&args[1].kind);

                let fn_loc = cx
                    .tcx
                    .hir()
                    .def_path(closure.def_id) 
                    .to_filename_friendly_no_crate(); 

                let fn_reviewer_id_check = check_identity(&correct_src_hash,
                                                                                    &(fn_reviewer_id.clone(),
                                                                                    fn_reviewer_src_signature.to_string()));
                if fn_reviewer_id_check.is_err() {
                    let mut help_msg = String::new();
                    push_id_error(&mut help_msg, "closure reviewer", &fn_reviewer_id_check);

                    if !Path::exists("./pcr/".as_ref()) {
                        fs::create_dir("./pcr/").unwrap();
                    }

                    if fn_reviewer_id_check.is_err() {
                        let pcr_file_name = format!("./pcr/{}.rs", fn_loc);
                        let src_hash_file_name = format!("./pcr/{}_src_hash.rs", fn_loc);
                        help_msg.push_str(
                            format!(
                                "wrote the hash of privacy-critical region into the file for signing: {}\n",
                                src_hash_file_name, 
                            ).as_str());
                        fs::write(pcr_file_name, pcr_src).unwrap();
                        fs::write(src_hash_file_name, correct_src_hash).unwrap();
                    }
                    span_lint_and_help(
                        cx,
                        ALOHOMORA_PCR,
                        expr.span,
                        "\x1b[93minvalid signature on privacy-critical region, could be a source of privacy policy violation bugs\x1b[0m",
                        None,
                        help_msg.as_str()
                    );
                }
            } else {
                panic!("Attempting to hash something other than a Closure.")
            }
        }
    }
}

/// Returns true if the given Expression is of ExprKind::Path & path resolves to given fn_path
fn is_fn_call(cx: &rustc_lint::LateContext, maybe_path: &Expr, fn_path: Vec<Symbol>) -> bool {
    if_chain!{
        if let ExprKind::Path(ref qpath) = maybe_path.kind;
        if let Res::Def(_, def_id) = cx.typeck_results().qpath_res(qpath, maybe_path.hir_id);
        then {
            cx.match_def_path(def_id, &fn_path)
        } else {
            false
        }    
    }
}

/// Given an ExprKind that should be a Signature struct, returns fields (username, signature)
fn extract_from_signature_struct(maybe_struct: &ExprKind) -> (String, String) {
    if let ExprKind::Struct(_, fields, _) = maybe_struct {
        assert!(fields.len() == 2);

        let username = if let ExprKind::Lit(spanned) = &fields[0].expr.kind {
            if let LitKind::Str(username, _) = spanned.node {
                String::from(username.as_str())
            } else {
                panic!("Attempting to use a non-string author username.");
            }
        } else {
            panic!("Attempting to use a non-string author username.");
        };

        let signature = if let ExprKind::Lit(spanned) = &fields[1].expr.kind {
            if let LitKind::Str(signature, _) = spanned.node {
                String::from(signature.as_str())
            } else {
                panic!("Attempting to use a non-string author username.");
            }
        } else {
            panic!("Attempting to use a non-string author username.");
        };

        (username, signature)
    } else {
        panic!("Invalid use of privacy-critical region.");
    }
}

/// Given a Closure, returns a hash 
/// of its in-crate source code (including helpers) + versions of invoked dependencies
fn get_pcr_hash<'a>(tcx: TyCtxt, closure: &rustc_hir::Closure) -> String {
    let def_id: rustc_hir::def_id::DefId = closure.def_id.to_def_id();

    // Instance of the signed closure to pass to Collector
    let instance = Instance::resolve( 
        tcx,
        ParamEnv::reveal_all(),
        def_id,
        InternalSubsts::identity_for_item(tcx, def_id),
    )
    .unwrap()
    .unwrap();

    let collector = Collector::collect(instance, tcx, true);
    let storage = collector.get_function_info_storage();
    let functions = storage.all();
 
    let mut src = vec![]; 
    let mut deps = HashSet::new(); 

    for function_info in functions.iter() {
        let instance = match function_info {
            FunctionInfo::WithBody { instance, .. } => 
                instance.to_owned(),
            FunctionInfo::WithoutBody { def_id, .. } => 
                panic!("this PCR calls into an unresolvable item at {:?}", tcx.def_path_debug_str(*def_id)),
        }; 
    
        let body: rustc_middle::mir::Body = instance
            .subst_mir_and_normalize_erasing_regions(
                tcx,
                ParamEnv::reveal_all(),
                tcx.instance_mir(function_info.instance().unwrap().def).to_owned(),
            );

        let src_snippet = tcx.sess
                            .source_map()
                            .span_to_snippet(body.span)
                            .unwrap();
        let normalized_snippet = normalize_source(src_snippet).unwrap(); 
        src.push(normalized_snippet); 

        deps.extend(compute_deps_for_body(body, tcx).into_iter());
    }
    // bind dependencies to AST hash
    let non_local_deps = deps.into_iter()
        .filter(|dep| 
            dep.clone() != tcx.crate_name(rustc_span::def_id::LOCAL_CRATE).to_string())
        .collect(); 
    let dep_strings = compute_dep_strings_for_crates(&non_local_deps);

    let mut src_hasher = Sha256::new();
    src.sort_unstable(); 
    src.into_iter().for_each(|snippet| src_hasher.update(snippet));
    dep_strings.into_iter().for_each(|(dep_string1, dep_string2 )| {
        let dep_string = format!("{} {}", dep_string1, dep_string2); 
        println!("dep_string: {}", dep_string);
        src_hasher.update(dep_string.as_str())
    } 
);
    let src_hash = hex::encode(&src_hasher.finalize()[..]); 

    src_hash
}

fn check_identity(target_plaintext: &String, identity: &(String, String)) -> Result<(), String> {
    let (username, signature) = identity;
    let keys = get_github_keys(username)
        .lines()
        .map(|line| format!("{}@github.com {}", username, line))
        .collect::<Vec<_>>()
        .join("\n");

    let decoded_signature_bytes = general_purpose::STANDARD_NO_PAD
        .decode(signature)
        .map_err(|err| err.to_string())?;
    let decoded_signature =
        std::str::from_utf8(decoded_signature_bytes.as_slice()).map_err(|err| err.to_string())?;

    fs::write("/tmp/allowed_signers", keys).map_err(|err| err.to_string())?;
    fs::write("/tmp/signature", decoded_signature).map_err(|err| err.to_string())?;
    fs::write("/tmp/plaintext", target_plaintext).map_err(|err| err.to_string())?;

    let github_url = format!("{}@github.com", username);
    let output = Command::new("/usr/bin/ssh-keygen")
        .args(["-Y", "verify", "-f", "/tmp/allowed_signers", "-I", &github_url, "-n", "file", "-s", "/tmp/signature"])
        .stdin(fs::File::open("/tmp/plaintext").map_err(|err| err.to_string())?)
        .output();

    fs::remove_file("/tmp/allowed_signers").map_err(|err| err.to_string())?;
    fs::remove_file("/tmp/signature").map_err(|err| err.to_string())?;
    fs::remove_file("/tmp/plaintext").map_err(|err| err.to_string())?;

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(())
            } else {
                Err(String::from(
                    std::str::from_utf8(output.stderr.as_slice()).map_err(|err| err.to_string())?,
                ))
            }
        }
        Err(err) => Err(err.to_string()),
    }
}

/// Given a String source code snippet, parses it to Exprs and Items, 
/// and returns as a String for hashing
fn normalize_source(code: String) -> Result<String, String> {
    if let Ok(expr) = parse_str::<syn::Expr>(code.as_str()) {
        let cleaned = expr.into_token_stream().to_string(); 
        return Ok(cleaned);
    }

    if let Ok(item) = parse_str::<syn::Item>(code.as_str()) {
        let cleaned = item.into_token_stream().to_string(); 
        return Ok(cleaned);
    }
    Err("failed to parse".to_string())
}

fn get_github_keys(username: &String) -> String {
    reqwest::blocking::get(format!("https://github.com/{}.keys", username))
        .unwrap()
        .text()
        .unwrap()
}

fn push_id_error(msg: &mut String, id: &str, res: &Result<(), String>) {
    if res.is_err() {
        msg.push_str(
            format!(
                "could not verify {}'s signature: {}\n",
                id,
                res.as_ref().unwrap_err().trim()
            )
            .as_str());
    }
}

#[test]
fn pcr_legal() {
    dylint_testing::ui_test_example(
        env!("CARGO_PKG_NAME"),
        "alohomora_pcr_legal"
    );
}

#[test]
fn pcr_illegal() {
    dylint_testing::ui_test_example(
        env!("CARGO_PKG_NAME"),
        "alohomora_pcr_illegal"
    );
}