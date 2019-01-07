#![feature(box_syntax)]
#![feature(rustc_private)]

extern crate rustc_driver;

use rustc::hir::intravisit::FnKind;
use syntax::ast::NodeId;
use syntax::source_map::Span;
use rustc_driver::driver;

#[macro_use] extern crate rustc;
extern crate syntax;
extern crate rustc_data_structures;

#[macro_use]
extern crate if_chain;

use rustc::lint::*;
use syntax::ast::Ident;

use rustc::hir::*;
use rustc::mir;
use rustc_data_structures::indexed_vec::IndexVec;
use rustc::ty;

use std::collections::{HashMap, HashSet};

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

declare_lint! {
    pub STATE_MACHINE,
    Forbid,
    "ensures that the code upholds the state machine models"
}

pub struct StateMachine {
    start: &'static str,
    transitions: HashMap<&'static str, (&'static str, &'static str)>,
}

impl LintPass for StateMachine {
    fn get_lints(&self) -> LintArray {
        lint_array!(STATE_MACHINE)
    }
}

impl<'a, 'tcx> LateLintPass<'a, 'tcx> for StateMachine {
    fn check_fn(
        &mut self,
        cx: &LateContext<'a, 'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl,
        body: &'tcx Body,
        _: Span,
        _: NodeId,
    ) {
        let def_id = cx.tcx.hir().body_owner_def_id(body.id());
        let mir = cx.tcx.optimized_mir(def_id);
        let mut states: IndexVec<mir::BasicBlock, HashSet<&'static str>> = IndexVec::from_elem(HashSet::new(), mir.basic_blocks());
        states[mir::START_BLOCK].insert(self.start);

        for (bb, bbdata) in mir.basic_blocks().iter_enumerated() {
            if let mir::TerminatorKind::Call { func, destination: Some((_, succ)), .. } = &bbdata.terminator().kind {
                let transition = self.transitions.iter().find(|(&transition, _)| {
                    match func.ty(&mir.local_decls, cx.tcx).sty {
                        ty::FnDef(did, _) => cx.tcx.item_name(did) == transition,
                        _ => false,
                    }
                });
                if let Some((transition, (start, end))) = transition {
                    if states[bb].contains(start) {
                        states[*succ].insert(end);
                    } else {
                        cx.span_lint(
                            STATE_MACHINE,
                            bbdata.terminator().source_info.span,
                            &format!("state transition `{}` not applicable for states {:?}", transition, states[bb] ),
                        );
                    }
                }
            } else {
                // the next block contains all states that the previous one did
                let new = states[bb].clone();
                for &succ in bbdata.terminator().successors() {
                    states[succ].extend(&new)
                }
            }
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

            let transitions = vec![
                ("begin_incoming_call", ("HangedUp", "Ringing")),
                ("accept_call", ("Ringing", "Calling")),
                ("end_call", ("Calling", "HangedUp")),
                ("begin_outgoing_call", ("HangedUp", "Dialing")),
                ("finished_dialing", ("Dialing", "Waiting")),
                ("call_accepted", ("Waiting", "Calling")),
            ];
            let transitions = transitions.into_iter().collect();
            ls.register_late_pass(None, false, box StateMachine { start: "HangedUp", transitions });
        });
        rustc_driver::run_compiler(&args, Box::new(compiler), None, None)
    });
}
