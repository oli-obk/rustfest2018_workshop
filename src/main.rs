#![feature(box_syntax)]
#![feature(rustc_private)]
#![feature(macro_vis_matcher)]

extern crate clippy_lints;
extern crate rustfest2018_workshop;

#[macro_use] extern crate rustc;
extern crate syntax;

use rustc::lint::*;
use syntax::ast::Ident;

declare_lint! {
    pub TRANSMUTE,
    Forbid,
    "the interns keep taking shortcuts that bite us later"
}

pub struct NoTransmute;

impl LintPass for NoTransmute {
    fn get_lints(&self) -> LintArray {
        lint_array!(TRANSMUTE)
    }
}

impl EarlyLintPass for NoTransmute {
    fn check_ident(&mut self, cx: &EarlyContext, ident: Ident) {
        if ident.to_string().contains("transmute") {
            cx.span_lint(
                TRANSMUTE,
                ident.span,
                "no. No. NO. NOOOOOO!!!! Like seriously, doesn't anyone read our coding guidelines?",
            );
        }
    }
}

fn main() {
    rustfest2018_workshop::run_lints(|ls| {
        ls.register_early_pass(None, false, box NoTransmute);
    });
}
