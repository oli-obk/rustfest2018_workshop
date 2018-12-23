#![feature(box_syntax)]
#![feature(rustc_private)]

extern crate rustc_driver;

use rustc_driver::driver;

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
            if let ExprKind::Binary(ref op, _, _) = expr.node;
            if BinOpKind::Div == op.node;
            then {
                let ty = cx.tables.expr_ty(expr);
                match ty.sty {
                    ty::Int(_) | ty::Uint(_) => {
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


pub fn main() {
    let args: Vec<_> = std::env::args().collect();
    rustc_driver::run(move || {
        let mut compiler = driver::CompileController::basic();
        compiler.after_parse.callback = Box::new(move |state| {
            let mut ls = state.session.lint_store.borrow_mut();
            ls.register_early_pass(None, false, box NoTransmute);
            ls.register_late_pass(None, false, box Pass);
        });
        rustc_driver::run_compiler(&args, Box::new(compiler), None, None)
    });
}
