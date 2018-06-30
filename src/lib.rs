#![feature(box_syntax)]
#![feature(rustc_private)]
#![allow(unknown_lints, missing_docs_in_private_items)]

extern crate getopts;
extern crate rustc;
extern crate rustc_driver;
extern crate rustc_errors;
extern crate rustc_plugin;
extern crate rustc_codegen_utils;
extern crate syntax;

use rustc_driver::{driver, Compilation};
use std::process::Command;
use rustc::lint::LintStore;
pub fn run_lints<F: Fn(&mut LintStore) + Send + Sync + 'static>(f: F) {
    use std::env;

    let sys_root = option_env!("SYSROOT")
        .map(String::from)
        .or_else(|| std::env::var("SYSROOT").ok())
        .or_else(|| {
            let home = option_env!("RUSTUP_HOME").or(option_env!("MULTIRUST_HOME"));
            let toolchain = option_env!("RUSTUP_TOOLCHAIN").or(option_env!("MULTIRUST_TOOLCHAIN"));
            home.and_then(|home| toolchain.map(|toolchain| format!("{}/toolchains/{}", home, toolchain)))
        })
        .or_else(|| {
            Command::new("rustc")
                .arg("--print")
                .arg("sysroot")
                .output()
                .ok()
                .and_then(|out| String::from_utf8(out.stdout).ok())
                .map(|s| s.trim().to_owned())
        })
        .expect("need to specify SYSROOT env var during compilation, or use rustup or multirust");

    // Setting RUSTC_WRAPPER causes Cargo to pass 'rustc' as the first argument.
    // We're invoking the compiler programmatically, so we ignore this/
    let mut orig_args: Vec<String> = env::args().collect();
    if orig_args.len() <= 1 {
        std::process::exit(1);
    }
    if orig_args[1] == "rustc" {
        // we still want to be able to invoke it normally though
        orig_args.remove(1);
    }
    // this conditional check for the --sysroot flag is there so users can call
    // `clippy_driver` directly
    // without having to pass --sysroot or anything
    let args: Vec<String> = if orig_args.iter().any(|s| s == "--sysroot") {
        orig_args.clone()
    } else {
        orig_args
            .clone()
            .into_iter()
            .chain(Some("--sysroot".to_owned()))
            .chain(Some(sys_root))
            .collect()
    };

    // this check ensures that dependencies are built but not linted and the final
    // crate is
    // linted but not built
    let clippy_enabled = orig_args.iter().any(|s| s == "--emit=dep-info,metadata") || std::env::var("LINTER_TESTMODE").is_ok();

    rustc_driver::run(move || {
        let mut compiler = driver::CompileController::basic();
        if clippy_enabled {
            let old = std::mem::replace(&mut compiler.after_parse.callback, box |_| {});
            compiler.after_parse.callback = Box::new(move |state| {
                f(&mut *state.session.lint_store.borrow_mut());
                old(state);
            });

            compiler.compilation_done.stop = Compilation::Stop;
        }
        rustc_driver::run_compiler(&args, Box::new(compiler), None, None)
    });
}
