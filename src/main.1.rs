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

fn main() {
    rustfest2018_workshop::run_lints(|ls| {
        ls.register_late_pass(None, false, box Pass);
    });
}
