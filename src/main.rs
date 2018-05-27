#![feature(box_syntax)]
#![feature(rustc_private)]
#![feature(macro_vis_matcher)]

extern crate clippy_lints;
extern crate rustfest2018_workshop;

#[macro_use] extern crate rustc;
extern crate syntax;

#[macro_use]
extern crate if_chain;

use rustc::lint::*;
use syntax::ast::Ident;

use rustc::hir::*;
use rustc::ty;

declare_lint! {
    pub NO_INT_DIV,
    Forbid,
    "integer division can panic, use checked_div"
}

pub struct Pass;

impl LintPass for Pass {
    fn get_lints(&self) -> LintArray {
        lint_array!(NO_INT_DIV)
    }
}

declare_lint! {
    pub TRANSMUTE,
    Forbid,
    "the interns keep taking shortcuts that bite us later"
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for Pass {
    fn check_expr(&mut self, cx: &LateContext<'a, 'tcx>, expr: &'tcx Expr) {
        if_chain! {
            if let Expr_::ExprBinary(ref op, _, _) = expr.node;
            if BinOp_::BiDiv == op.node;
            then {
                let ty = cx.tables.expr_ty(expr);
                match ty.sty {
                    ty::TyInt(_) | ty::TyUint(_) => {
                        cx.span_lint(
                            NO_INT_DIV,
                            expr.span,
                            "This might panic",
                        );
                    },
                    _ => {},
                }
            }
        }
    }
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
        ls.register_late_pass(None, false, box Pass);
    });
}
