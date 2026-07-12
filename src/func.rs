//! Machinery for defining and exporting functions to the Lisp runtime. It should be mainly used by
//! the #[[`defun`]] macro, not module code.
//!
//! [`defun`]: attr.defun.html

use {
    crate::{Env, Value, Result, FromLisp, IntoLisp, subr},
    std::{
        os, panic,
        ffi::{c_void, CStr, CString},
        ops::{Deref, Range},
        ptr, slice,
    },
    emacs_module::{emacs_env, emacs_value, EmacsSubr, emacs_variadic_function},
};

pub trait Manage {
    unsafe fn make_function<T: Into<Vec<u8>>>(
        &self,
        function: EmacsSubr,
        arities: Range<usize>,
        doc: T,
        data: *mut os::raw::c_void,
    ) -> Result<Value<'_>>;

    fn fset(&self, name: &str, func: Value<'_>) -> Result<Value<'_>>;
}

impl Manage for Env {
    /// # Safety
    ///
    /// The `function` must use `data` pointer in safe ways.
    #[allow(unused_unsafe)]
    unsafe fn make_function<T: Into<Vec<u8>>>(
        &self,
        function: EmacsSubr,
        arities: Range<usize>,
        doc: T,
        data: *mut os::raw::c_void,
    ) -> Result<Value<'_>> {
        unsafe_raw_call_value!(
            self,
            make_function,
            arities.start as isize,
            arities.end as isize,
            Some(function),
            CString::new(doc)?.as_ptr(),
            data
        )
    }

    fn fset(&self, name: &str, func: Value<'_>) -> Result<Value<'_>> {
        let symbol = self.intern(name)?;
        self.call("fset", [symbol, func])
    }
}

/// Like [`Env`], but is available only in exported functions. This has additional methods to handle
/// arguments passed from Lisp code.
///
/// [`Env`]: struct.Env.html
#[doc(hidden)]
#[derive(Debug)]
pub struct CallEnv {
    env: Env,
    nargs: usize,
    args: *mut emacs_value,
}

// TODO: Iterator and Index
impl CallEnv {
    #[doc(hidden)]
    #[inline]
    pub unsafe fn new(env: Env, nargs: isize, args: *mut emacs_value) -> Self {
        let nargs = nargs as usize;
        Self { env, nargs, args }
    }

    #[doc(hidden)]
    #[inline]
    pub fn raw_args(&self) -> &[emacs_value] {
        // Safety: Emacs assures *args is valid for the duration of the call, with length nargs.
        unsafe { slice::from_raw_parts(self.args, self.nargs) }
    }

    pub fn args(&self) -> Vec<Value<'_>> {
        // Safety: Emacs assures *args are on the stack for the duration of the call.
        self.raw_args().iter().map(|v| unsafe { Value::new(*v, &self.env) }).collect()
    }

    #[inline]
    pub fn get_arg(&self, i: usize) -> Value<'_> {
        let args: &[emacs_value] = self.raw_args();
        // Safety: Emacs assures *args are on the stack for the duration of the call.
        unsafe { Value::new(args[i], self) }
    }

    #[inline]
    pub fn parse_arg<'e, T: FromLisp<'e>>(&'e self, i: usize) -> Result<T> {
        self.get_arg(i).into_rust(&self.env)
    }
}

/// This allows `Env`'s methods to be called on a `CallEnv`.
impl Deref for CallEnv {
    type Target = Env;

    #[doc(hidden)]
    #[inline(always)]
    fn deref(&self) -> &Env {
        &self.env
    }
}

pub trait HandleCall {
    fn handle_call<'e, T, F>(&'e self, f: F) -> emacs_value
    where
        F: Fn(&'e CallEnv) -> Result<T> + panic::RefUnwindSafe,
        T: IntoLisp<'e>;
}

impl HandleCall for CallEnv {
    #[inline]
    fn handle_call<'e, T, F>(&'e self, f: F) -> emacs_value
    where
        F: Fn(&'e CallEnv) -> Result<T> + panic::RefUnwindSafe,
        T: IntoLisp<'e>,
    {
        let env = panic::AssertUnwindSafe(self);
        let result = panic::catch_unwind(|| unsafe {
            let rust_result = f(&env);
            let lisp_result = rust_result.and_then(|t| t.into_lisp(&env));
            env.maybe_exit(lisp_result)
        });
        env.handle_panic(result)
    }
}

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
