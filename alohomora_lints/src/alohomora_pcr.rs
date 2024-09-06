#![warn(unused_extern_crates)]

extern crate rustc_ast;
extern crate rustc_data_structures;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_query_system;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_help;
use clippy_utils::sym;

use rustc_ast::ast::LitKind;
use rustc_span::symbol::Symbol;

use rustc_hir::def::Res;
use rustc_hir::Expr;
use rustc_hir::ExprKind;

use rustc_middle::ty::{subst::InternalSubsts, Instance, ParamEnv, TyCtxt};

use rustc_data_structures::stable_hasher::{HashStable, StableHasher};
use rustc_query_system::ich::StableHashingContext;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use base64::{engine::general_purpose, Engine as _};
use if_chain::if_chain;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::collections::HashSet;
//use std::time::{SystemTime, UNIX_EPOCH};

use scrutils::Collector;
use scrutils::compute_deps_for_body; 
use scrutils::compute_dep_strings_for_crates;

declare_alohomora_lint! {
    /// ### What it does
    /// Warns if PrivacyCriticalRegions have invalid signatures. 
    /// 
    /// ### Why is this bad?
    /// Closures in PrivacyCriticalRegions must be signed to indicate they have been 
    /// reviewed and do not pose privacy risks. 
    /// 
    /// An invalidated signature indicates that the closure or a function
    /// the closure calls has changed since the last signature.
    /// 
    /// ### Known problems
    /// Functions from external crates called within the PCR are not included in the hash of the closure, 
    /// so changes in an external crate will not invalidate the signature. 
    /// 
    /// ### Example
    /// ```rust
    /// //  let pcr = PrivacyCriticalRegion::new(|x: u8| { <privacy-critical closure> },
    /// //          Signature {username: "corinnt", 
    /// //              signature: "LS0tLS....."},     // author signature on closure
    /// //          Signature {username: "corinnt", 
    /// //              signature: "LS0tLS....."},     // review signature on closure
    /// //          Signature {username: "corinnt", 
    /// //              signature: "LK0tLM..."})       // signature on Cargo.lock
    /// ```
    pub ALOHOMORA_PCR,
    Warn,
    "highlights and hashes privacy-critical regions", 
    check_expr(cx: &rustc_lint::LateContext<'tcx>, expr: &'_ rustc_hir::Expr<'_>)
}

fn check_expr<'tcx>(cx: &rustc_lint::LateContext<'tcx>, expr: &'_ rustc_hir::Expr<'_>) {
    let target_fn_path: Vec<Symbol> = vec![
        sym!(alohomora),
        sym!(pcr),
        sym!(PrivacyCriticalRegion),
        sym!(new),
    ];

    if let ExprKind::Call(maybe_path, args) = &expr.kind {
        if is_fn_call(cx, maybe_path, target_fn_path) {
            assert!(args.len() == 4); // 4 args to constructor of PrivacyCriticalRegion
            if let ExprKind::Closure(closure) = args[0].kind {
                let closure_body = cx.tcx.hir().body(closure.body);
                let pcr_src = cx
                    .tcx
                    .sess
                    .source_map()
                    .span_to_snippet(closure_body.value.span)
                    .unwrap();
                let cargo_lock_hash = get_cargo_lock_hash(cx.tcx); 
                let (correct_src_hash, correct_mir_hash) = get_pcr_hashes(cx.tcx, closure);
                
                //These args to PrivacyCriticalRegion::new will be of type Signature
                let (author_id, author_full_signature ) = extract_from_signature_struct(&args[1].kind);
                let (fn_reviewer_id, fn_reviewer_full_signature ) = extract_from_signature_struct(&args[2].kind);
                let dep_reviewer = extract_from_signature_struct(&args[3].kind);

                let fn_loc = cx
                    .tcx
                    .hir()
                    .def_path(closure.def_id) 
                    .to_filename_friendly_no_crate(); 

                let (author_src_signature, author_mir_signature) = author_full_signature
                                                            .split_once("#")
                                                            .ok_or(format!("sign pcr @ {} with sign_pcr.sh", fn_loc)).unwrap(); 

                let (fn_reviewer_src_signature, fn_reviewer_mir_signature) : (&str, &str) = fn_reviewer_full_signature
                                                            .split_once("#")
                                                            .ok_or(format!("sign pcr @ {} with sign_pcr.sh", fn_loc)).unwrap(); 

                let author_id_src_check = check_identity(&correct_src_hash, &(author_id.clone(), author_src_signature.to_string()));
                let author_id_mir_check = check_identity(&correct_mir_hash, &(author_id, author_mir_signature.to_string()));
                println!("author src {:?}", author_id_src_check); 
                println!("author mir {:?}", author_id_mir_check);
                let author_id_check = author_id_src_check.or(author_id_mir_check); 
               

                let fn_reviewer_id_src_check = check_identity(&correct_src_hash, &(fn_reviewer_id.clone(), fn_reviewer_src_signature.to_string()));
                let fn_reviewer_id_mir_check = check_identity(&correct_src_hash, &(fn_reviewer_id, fn_reviewer_mir_signature.to_string()));
                let fn_reviewer_id_check = fn_reviewer_id_src_check.or(fn_reviewer_id_mir_check); 

                let dep_reviewer_id_check = check_identity(&cargo_lock_hash, &dep_reviewer);

                if author_id_check.is_err() 
                    || fn_reviewer_id_check.is_err()
                    || dep_reviewer_id_check.is_err() {

                    let mut help_msg = String::new();
                    push_id_error(&mut help_msg, "Cargo.lock reviewer", &dep_reviewer_id_check);
                    push_id_error(&mut help_msg, "author", &author_id_check);
                    push_id_error(&mut help_msg, "closure reviewer", &fn_reviewer_id_check);

                    if !Path::exists("./pcr/".as_ref()) {
                        fs::create_dir("./pcr/").unwrap();
                    }
                    if dep_reviewer_id_check.is_err(){
                        let cargo_lock_file_name = format!("./pcr/Cargo.lock_hash"); // _{}", timestamp);
                        help_msg.push_str(
                            format!("written the hash of Cargo.lock into the file for signing: {}\n",
                                    cargo_lock_file_name
                            ).as_str());
                        fs::write(cargo_lock_file_name, cargo_lock_hash).unwrap();
                    } 
                    if author_id_check.is_err() || fn_reviewer_id_check.is_err() {
                        let pcr_file_name = format!("./pcr/{}.rs", fn_loc);
                        let src_hash_file_name = format!("./pcr/{}_src_hash.rs", fn_loc);
                        let mir_hash_file_name = format!("./pcr/{}_mir_hash.rs", fn_loc);
                        help_msg.push_str(
                            format!(
                                "written the hash of privacy-critical region into the files for signing: {} {}\n",
                                src_hash_file_name, 
                                mir_hash_file_name
                            ).as_str());
                        fs::write(pcr_file_name, pcr_src).unwrap();
                        fs::write(src_hash_file_name, correct_src_hash).unwrap();
                        fs::write(mir_hash_file_name, correct_mir_hash).unwrap();
                    }
                    span_lint_and_help(
                        cx,
                        ALOHOMORA_PCR,
                        expr.span,
                        "\x1b[93mbadly-signed privacy-critical region, might be a source of privacy-related bugs\x1b[0m",
                        None,
                        help_msg.as_str()
                    );
                }
            } else {
                panic!("Attempting to hash something different from a Closure.")
            }
        }
    }
}

/// Returns true if the given Expression is of ExprKind::Path & path resolves to given fn_pat
fn is_fn_call(cx: &rustc_lint::LateContext, maybe_path: &Expr, fn_path: Vec<Symbol>) -> bool {
    if_chain!{
        if let ExprKind::Path(ref qpath) = maybe_path.kind;
        if let Res::Def(_kind, def_id) = cx.typeck_results().qpath_res(qpath, maybe_path.hir_id);
        then {
            cx.match_def_path(def_id, &fn_path)
        } else {
            false
        }    
    }
}

/// Given an ExprKind that may be a Signature struct, returns fields (username, signature)
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

/// Recursively finds the path to the innermost Cargo.lock file
fn get_cargo_lock(directory: PathBuf) -> Result<PathBuf, String> { 
    let cargo_lock_path = directory.join("Cargo.lock");
    if cargo_lock_path.is_file() {
        Ok(cargo_lock_path) 
    } else {
        match directory.parent() {
            Some(parent_dir) => {
                assert!(parent_dir != directory); 
                get_cargo_lock(parent_dir.to_path_buf())
            }, 
            None => Err(format!("No Cargo.lock found in {}", directory.display()))
        }
    }
}

/// Given the lint pass's TyCtxt, 
///   returns the StableHash of the contents of the Cargo.lock of the cwd 
fn get_cargo_lock_hash(tcx: TyCtxt) -> String {
    let cwd = std::env::current_dir().unwrap(); 
    let toml_path = get_cargo_lock(cwd).unwrap(); 
    //println!("TOML PATH {}", toml_path.display()); 
    let toml_contents = fs::read_to_string(toml_path).unwrap(); 
    
    let mut hcx = StableHashingContext::new(tcx.sess, tcx.untracked());
    let mut hasher = StableHasher::new();
    toml_contents.hash_stable(&mut hcx, &mut hasher);

    let hash_tuple: (u64, u64) = hasher.finalize();
    let toml_hash = format!("{:x} {:x}", hash_tuple.0, hash_tuple.1);
    toml_hash
}  

/// Given a Closure, returns a tuple of the hash of the source code and a StableHash of its MIR. 
fn get_pcr_hashes<'a>(tcx: TyCtxt, closure: &rustc_hir::Closure) -> (String, String) {
    let def_id: rustc_hir::def_id::DefId = closure.def_id.to_def_id();

    // instance of the parent signed closure to pass to Collector
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

    let mut hcx = StableHashingContext::new(tcx.sess, tcx.untracked());
    let mut mir_hasher = StableHasher::new();   
    let mut src = vec![]; 
    let mut deps = HashSet::new(); 

    for function_info in functions.iter() {
        let instance = function_info.instance().unwrap();
        //let def_id = instance.def_id();
        let body: rustc_middle::mir::Body = instance
            .subst_mir_and_normalize_erasing_regions(
                tcx,
                ParamEnv::reveal_all(),
                tcx.instance_mir(function_info.instance().unwrap().def).to_owned(),
            );
        body.hash_stable(&mut hcx, &mut mir_hasher);

        let src_snippet = tcx.sess
                            .source_map()
                            .span_to_snippet(body.span)
                            .unwrap();
        src.push(src_snippet); 

        deps.extend(compute_deps_for_body(body, tcx).into_iter());
    }
    // goal here is to bind to deps to MIR hash
    //TODO this is currently adding all the dependencies, not just the local ones, 
    // which should be false_positives v0.1.0 and dependency v0.2.0 for `pcr_examples/false_positives`
    let non_local_deps = deps.into_iter()
        .filter(|dep| dep.clone() != tcx.crate_name(rustc_span::def_id::LOCAL_CRATE).to_string())
        .collect(); 
    let dep_strings = compute_dep_strings_for_crates(&non_local_deps);
    println!("dep_strings: {:#?}", dep_strings); 
    dep_strings.iter().for_each(|dep_string| dep_string.hash_stable(&mut hcx, &mut mir_hasher));
    let mir_hash_tuple: (u64, u64) = mir_hasher.finalize();
    let mir_hash = format!("{:x} {:x}", mir_hash_tuple.0, mir_hash_tuple.1);

    let mut src_hasher = DefaultHasher::new();
    src.sort_unstable(); 
    src.into_iter().for_each(|snippet| snippet.hash(&mut src_hasher));
    dep_strings.into_iter().for_each(|dep_string| dep_string.hash(&mut src_hasher));
    let src_hash: String = src_hasher.finish().to_string(); 

    (src_hash, mir_hash)
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

    let command_str = format!("/usr/bin/ssh-keygen -Y verify -f /tmp/allowed_signers -I {}@github.com -n file -s /tmp/signature < /tmp/plaintext", username);

    let mut command = Command::new("zsh");
    command.args(["-c", command_str.as_str()]);
    let output = command.output();

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
fn alohomora_pcr_basic_call_legal() {
    dylint_testing::ui_test_example(
        env!("CARGO_PKG_NAME"),
        "alohomora_pcr_basic_call_legal"
    );
}

// Manual tests / examples in separate crate