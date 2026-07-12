//! Machinery for defining and exporting functions to the Lisp runtime. It should be mainly used by
//! the #[[`defun`]] macro, not module code.
//!
//! [`defun`]: attr.defun.html

use {
    crate::{Env, Value, Result, IntoLisp, subr},
    std::{
        panic,
        ffi::{c_void, CStr},
        ptr, slice,
    },
    emacs_module::{emacs_env, emacs_value, emacs_variadic_function},
};

fn handle_call<'e, F, T>(env: &'e Env, f: F) -> emacs_value
where
    F: Fn(&'e Env) -> Result<T> + panic::RefUnwindSafe,
    T: IntoLisp<'e>,
{
    let env = panic::AssertUnwindSafe(env);
    env.handle_panic(panic::catch_unwind(|| {
        let val = f(&env).and_then(|val| val.into_lisp(&env));
        unsafe { env.maybe_exit(val) }
    }))
}

pub trait LispFn<'e> {
    const MIN_ARITY: usize;
    const MAX_ARITY: Option<usize>;

    fn call(&self, _: &'e Env, args: &[Value<'e>]) -> Result<Value<'e>>;
}
#[doc(hidden)]
pub unsafe extern "C" fn extern_lambda<F>(
    env: *mut emacs_env,
    nargs: isize,
    args: *mut emacs_value,
    data: *mut c_void,
) -> emacs_value
where
    F: for<'e> LispFn<'e>,
{
    let env = unsafe { Env::new(env) };
    handle_call(&env, |env| {
        let len = usize::try_from(nargs).unwrap_or_default();
        let args = if len == 0 { &[] } else { unsafe { slice::from_raw_parts(args.cast(), len) } };

        let f = data.cast::<F>();
        let f = unsafe { f.as_ref() }.unwrap();
        f.call(env, args)
    })
}

pub struct Lambda<'e>(Value<'e>);
impl<'e> Lambda<'e> {
    pub fn fset(&self, env: &'e Env, name: &str) -> Result<()> {
        env.call(&subr::FSET, (env.intern(name)?, self.0))?;
        Ok(())
    }
}
impl<'e> IntoLisp<'e> for Lambda<'e> {
    fn into_lisp(self, _: &'e Env) -> Result<Value<'e>> {
        Ok(self.0)
    }
}

impl Env {
    pub fn lambda<'e, F>(
        &'e self,
        f: &'static F,
        docstring: Option<&'static CStr>,
    ) -> Result<Lambda<'e>>
    where
        F: for<'env> LispFn<'env>,
    {
        unsafe_raw_call_value!(
            self,
            make_function,
            F::MIN_ARITY.try_into().unwrap(),
            F::MAX_ARITY
                .map(|max| max.try_into().unwrap())
                .unwrap_or(emacs_variadic_function.try_into().unwrap()),
            Some(extern_lambda::<F>),
            docstring.map(CStr::as_ptr).unwrap_or_default(),
            ptr::from_ref(f).cast_mut().cast()
        )
        .map(Lambda)
    }
}
