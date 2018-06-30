#![feature(box_syntax)]
#![feature(rustc_private)]

extern crate rustc;
extern crate rustc_driver;

use rustc_driver::{driver, Compilation};
use rustc::lint::LintStore;
pub fn run_lints<F: Fn(&mut LintStore) + Send + Sync + 'static>(f: F) {
    let args: Vec<_> = std::env::args().collect();
    // this check ensures that dependencies are built but not linted and the final
    // crate is
    // linted but not built
    let clippy_enabled = args.iter().any(|s| s == "--emit=dep-info,metadata") || std::env::var("LINTER_TESTMODE").is_ok();

    rustc_driver::run(move || {
        let mut compiler = driver::CompileController::basic();
        if clippy_enabled {
            compiler.after_parse.callback = Box::new(move |state| {
                f(&mut *state.session.lint_store.borrow_mut());
            });

            compiler.compilation_done.stop = Compilation::Stop;
        }
        rustc_driver::run_compiler(&args, Box::new(compiler), None, None)
    });
}
