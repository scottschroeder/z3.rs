use z3_sys::*;
use Context;
use Sort;
use Symbol;
use Ast;
use Pattern;
use Z3_MUTEX;

use std::hash::{Hash, Hasher};
use std::cmp::{Eq, PartialEq};
use std::ffi::CString;

impl<'ctx> Pattern<'ctx> {
    pub fn new(ctx: &Context, func: Z3_func_decl) -> Pattern {
        assert!(!func.is_null());
        Pattern {
            ctx,
            z3_func: unsafe {
                debug!("new func {:p}", func);
                let guard = Z3_MUTEX.lock().unwrap();
                Z3_inc_ref(ctx.z3_ctx, Z3_func_decl_to_ast(ctx.z3_ctx, func));
                func
            },
        }
    }

}


impl<'ctx> Drop for Pattern<'ctx> {
    fn drop(&mut self) {
        unsafe {
            debug!("drop ast {:p}", self.z3_func);
            let guard = Z3_MUTEX.lock().unwrap();
            Z3_dec_ref(self.ctx.z3_ctx, Z3_func_decl_to_ast(self.ctx.z3_ctx, self.z3_func));
        }
    }
}

impl<'ctx> Hash for Pattern<'ctx> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            let u = Z3_get_ast_hash(self.ctx.z3_ctx, Z3_func_decl_to_ast(self.ctx.z3_ctx, self.z3_func));
            u.hash(state);
        }
    }
}

impl<'ctx> PartialEq<Pattern<'ctx>> for Pattern<'ctx> {
    fn eq(&self, other: &Pattern<'ctx>) -> bool {
        unsafe {
            Z3_TRUE == Z3_is_eq_func_decl(
                self.ctx.z3_ctx,
                self.z3_func,
                other.z3_func,
            )
        }
    }
}

impl<'ctx> Eq for Pattern<'ctx> {}
