use z3_sys::*;
use Context;
use Sort;
use Symbol;
use Ast;
use Function;
use Z3_MUTEX;

use std::hash::{Hash, Hasher};
use std::cmp::{Eq, PartialEq};
use std::ffi::{CString, CStr};

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

    /// Create an axiom to make this function injective over the i-th argument
    ///
    /// ```
    /// forall (x_0, ..., x_n) f_inv(f(x_0, ..., x_i, ..., x_{n-1})) = x_i
    ///
    /// where `f_inv` is a fresh function declaration meaning the inverse of `f`
    ///
    /// Based on: https://github.com/Z3Prover/z3/blob/450da5ea0c93719c63ef3629149d10e5e2f58dac/examples/c/test_capi.c#L306
    pub fn injective_axiom(&self, i_arg: usize) -> Ast<'ctx> {
        Ast::new(self.ctx, unsafe {
            let guard = Z3_MUTEX.lock().unwrap();
            let z3ctx = self.ctx.z3_ctx;
            let domain_size = Z3_get_domain_size(z3ctx, self.z3_func);
            if i_arg >= domain_size as usize {
                panic!("It is invalid to make a function of arity {} injective over its {}-th arg", domain_size, i_arg)
            }

            let finv_domain = Z3_get_range(z3ctx, self.z3_func);
            let finv_range = Z3_get_domain(z3ctx, self.z3_func, i_arg as u32);
            let finv = Z3_mk_fresh_func_decl(
                z3ctx,
                CString::new("inv").unwrap().as_ptr(),
                1,
                &finv_domain,
                finv_range,
            );
            Z3_inc_ref(z3ctx, Z3_func_decl_to_ast(z3ctx, finv));

            let mut types: Vec<Z3_sort> = Vec::with_capacity(domain_size as usize);
            let mut names: Vec<Z3_symbol> = Vec::with_capacity(domain_size as usize);
            let mut args: Vec<Z3_ast> = Vec::with_capacity(domain_size as usize);

            for i in 0..domain_size {
                let i_sort = Z3_get_domain(z3ctx, self.z3_func, i);
                types.push(i_sort);
                names.push(Z3_mk_int_symbol(z3ctx, i as i32));
                let arg = Z3_mk_bound(z3ctx, i, i_sort);
                Z3_inc_ref(z3ctx, arg);
                args.push(arg);
            }


            let fxs = Z3_mk_app(z3ctx, self.z3_func, domain_size, args.as_ptr());
            Z3_inc_ref(z3ctx, fxs);
            let finv_fxs = Z3_mk_app(z3ctx, finv, 1, [fxs].as_ptr());
            Z3_inc_ref(z3ctx, finv_fxs);
            let equality = Z3_mk_eq(z3ctx, finv_fxs, args[i_arg]);
            Z3_inc_ref(z3ctx, equality);

            let pattern = Z3_mk_pattern(z3ctx, 1, &fxs);
            let quantifier = Z3_mk_forall(
                z3ctx,
                0,
                1,
                &pattern,
                domain_size,
                types.as_ptr(),
                names.as_ptr(),
                equality,
            );
            Z3_dec_ref(z3ctx, finv_fxs);
            Z3_dec_ref(z3ctx, fxs);
            for arg in args {
                Z3_dec_ref(z3ctx, arg);
            }
            Z3_dec_ref(z3ctx, Z3_func_decl_to_ast(z3ctx, finv));
            Z3_dec_ref(z3ctx, equality);
            quantifier
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
