use z3_sys::*;
use Context;
use Sort;
use Symbol;
use Ast;
use Function;
use Z3_MUTEX;

use std::hash::{Hash, Hasher};
use std::cmp::{Eq, PartialEq};
use std::ffi::CString;

impl<'ctx> Function<'ctx> {
    pub fn new(ctx: &Context, func: Z3_func_decl) -> Function {
        assert!(!func.is_null());
        Function {
            ctx,
            z3_func: unsafe {
                debug!("new func {:p}", func);
                let guard = Z3_MUTEX.lock().unwrap();
                Z3_inc_ref(ctx.z3_ctx, Z3_func_decl_to_ast(ctx.z3_ctx, func));
                func
            },
        }
    }

    pub fn new_decl(sym: &Symbol<'ctx>, domain: &[&Sort<'ctx>], range: &Sort<'ctx>) -> Function<'ctx> {
        let domain_ptrs: Vec<Z3_sort> = domain.iter().map(|s| s.z3_sort).collect();

        Function::new(sym.ctx, unsafe {
            let guard = Z3_MUTEX.lock().unwrap();
            Z3_mk_func_decl(
                sym.ctx.z3_ctx,
                sym.z3_sym,
                domain_ptrs.len() as ::std::os::raw::c_uint,
                domain_ptrs.as_ptr(),
                range.z3_sort,
            )
        })
    }

    pub fn fresh_decl(prefix: &str, domain: &[&Sort<'ctx>], range: &Sort<'ctx>) -> Function<'ctx> {
        let domain_ptrs: Vec<Z3_sort> = domain.iter().map(|s| s.z3_sort).collect();
        let pp = CString::new(prefix).unwrap();
        let p = pp.as_ptr();

        Function::new(range.ctx, unsafe {
            let guard = Z3_MUTEX.lock().unwrap();
            Z3_mk_fresh_func_decl(
                range.ctx.z3_ctx,
                p,
                domain_ptrs.len() as ::std::os::raw::c_uint,
                domain_ptrs.as_ptr(),
                range.z3_sort,
            )
        })
    }

    pub fn call(&self, args: &[&Ast<'ctx>]) -> Ast<'ctx> {
        let arg_ptrs: Vec<Z3_ast> = args.iter().map(|s| s.z3_ast).collect();

        Ast::new(self.ctx, unsafe {
            let guard = Z3_MUTEX.lock().unwrap();
            Z3_mk_app(
                self.ctx.z3_ctx,
                self.z3_func,
                arg_ptrs.len() as ::std::os::raw::c_uint,
                arg_ptrs.as_ptr(),
            )
        })
    }

    // Functions which may be useful to implement
    // Z3_get_func_decl_id
    // Z3_get_decl_name
    // Z3_get_domain_size
    // Z3_get_domain
    // Z3_get_range
}


impl<'ctx> Drop for Function<'ctx> {
    fn drop(&mut self) {
        unsafe {
            debug!("drop ast {:p}", self.z3_func);
            let guard = Z3_MUTEX.lock().unwrap();
            Z3_dec_ref(self.ctx.z3_ctx, Z3_func_decl_to_ast(self.ctx.z3_ctx, self.z3_func));
        }
    }
}

impl<'ctx> Hash for Function<'ctx> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        unsafe {
            let u = Z3_get_ast_hash(self.ctx.z3_ctx, Z3_func_decl_to_ast(self.ctx.z3_ctx, self.z3_func));
            u.hash(state);
        }
    }
}

impl<'ctx> PartialEq<Function<'ctx>> for Function<'ctx> {
    fn eq(&self, other: &Function<'ctx>) -> bool {
        unsafe {
            Z3_TRUE == Z3_is_eq_func_decl(
                self.ctx.z3_ctx,
                self.z3_func,
                other.z3_func,
            )
        }
    }
}

impl<'ctx> Eq for Function<'ctx> {}
