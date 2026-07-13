#[doc(no_inline)]
use std::{
    any::Any,
    fmt::{self, Display, Formatter},
    mem::MaybeUninit,
    result, thread,
};

use emacs_module::*;

use crate::{
    Env, Value, IntoLisp,
    global::{LazyGlobalRef, Symbol},
    symbol::{self, IntoLispSymbol},
    call::IntoLispArgs,
};

// We use const instead of enum, in case Emacs add more exit statuses in the future.
// See https://github.com/rust-lang/rust/issues/36927
pub(crate) const RETURN: emacs_funcall_exit = emacs_funcall_exit_return;
pub(crate) const SIGNAL: emacs_funcall_exit = emacs_funcall_exit_signal;
pub(crate) const THROW: emacs_funcall_exit = emacs_funcall_exit_throw;

pub type Error = Box<dyn std::error::Error>;

#[derive(Debug)]
pub struct TempValue {
    raw: emacs_value,
}

/// Error types generic to all Rust dynamic modules.
///
/// This list is intended to grow over time and it is not recommended to exhaustively match against
/// it.
#[derive(Debug)]
pub enum ErrorKind {
    /// An [error] signaled by Lisp code.
    ///
    /// [error]: https://www.gnu.org/software/emacs/manual/html_node/elisp/Signaling-Errors.html
    Signal { symbol: TempValue, data: TempValue },

    /// A [non-local exit] thrown by Lisp code.
    ///
    /// [non-local exit]: https://www.gnu.org/software/emacs/manual/html_node/elisp/Catch-and-Throw.html
    Throw { tag: TempValue, value: TempValue },

    /// An error indicating that the given value is not a `user-ptr` of the expected type.
    ///
    /// # Examples:
    ///
    /// ```
    /// # use rem::*;
    /// # use std::cell::RefCell;
    /// #[defun(name = Wrap)]
    /// fn wrap(x: i64) -> Result<RefCell<i64>> {
    ///     Ok(RefCell::new(x))
    /// }
    ///
    /// #[defun(name = WrapF)]
    /// fn wrap_f(x: f64) -> Result<RefCell<f64>> {
    ///     Ok(RefCell::new(x))
    /// }
    ///
    /// #[defun(name = Unwrap)]
    /// fn unwrap(r: &RefCell<i64>) -> Result<i64> {
    ///     Ok(*r.try_borrow()?)
    /// }
    ///
    /// #[module]
    /// fn init(env: &rem::Env) -> Result<()> {
    ///     env.lambda(&Wrap, None)?.fset("wrap")?;
    ///     env.lambda(&WrapF, None)?.fset("wrap-f")?;
    ///     env.lambda(&Unwrap, None)?.fset("unwrap")?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ```emacs-lisp
    /// (unwrap 7)          ; *** Eval error ***  Wrong type argument: user-ptrp, 7
    /// (unwrap (wrap 7))   ; 7
    /// (unwrap (wrap-f 7)) ; *** Eval error ***  Wrong type user-ptr: "expected: RefCell"
    /// ```
    WrongTypeUserPtr { expected: &'static str },
}
impl Display for ErrorKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Signal { symbol, data } => {
                write!(f, "Non-local signal: symbol={symbol:?} data={data:?}")
            }
            Self::Throw { tag, value } => {
                write!(f, "Non-local throw: tag={tag:?} value={value:?}")
            }
            Self::WrongTypeUserPtr { expected } => {
                write!(f, "expected: {expected}")
            }
        }
    }
}
impl std::error::Error for ErrorKind {}

/// A specialized [`Result`] type for Emacs's dynamic modules.
///
/// [`Result`]: https://doc.rust-lang.org/std/result/enum.Result.html
pub type Result<T> = result::Result<T, Error>;

// FIX: Make this into RootedValue (or ProtectedValue), and make it safe. XXX: The problem is that
// the raw value will be leaked when RootedValue is dropped, since `free_global_ref` requires an env
// (thus cannot be called there). This is likely a mis-design in Emacs (In Erlang,
// `enif_keep_resource` and `enif_release_resource` don't require an env).
impl TempValue {
    unsafe fn new(raw: emacs_value) -> Self {
        Self { raw }
    }

    /// # Safety
    ///
    /// This must only be used with the [`Env`] from which the error originated.
    ///
    /// [`Env`]: struct.Env.html
    pub unsafe fn value<'e>(&self, env: &'e Env) -> Value<'e> {
        // SAFETY: Caller guarantees env is the Env from which this error originated.
        unsafe { Value::new(self.raw, env) }.protect(env)
    }
}

// XXX: Technically these are unsound, but they are necessary to use the `Fail` trait. We ensure
// safety by marking TempValue methods as unsafe.
unsafe impl Send for TempValue {}

unsafe impl Sync for TempValue {}

impl Env {
    /// Handles possible non-local exit after calling Lisp code.
    #[inline]
    pub(crate) fn handle_exit<T>(&self, result: T) -> Result<T> {
        let mut symbol = MaybeUninit::uninit();
        let mut data = MaybeUninit::uninit();
        // TODO: Check whether calling non_local_exit_check first makes a difference in performance.
        let status = self.non_local_exit_get(&mut symbol, &mut data);
        match (status, symbol, data) {
            (RETURN, ..) => Ok(result),
            (SIGNAL, symbol, data) => {
                self.non_local_exit_clear();
                Err(ErrorKind::Signal {
                    symbol: unsafe { TempValue::new(symbol.assume_init()) },
                    data: unsafe { TempValue::new(data.assume_init()) },
                }
                .into())
            }
            (THROW, tag, value) => {
                self.non_local_exit_clear();
                Err(ErrorKind::Throw {
                    tag: unsafe { TempValue::new(tag.assume_init()) },
                    value: unsafe { TempValue::new(value.assume_init()) },
                }
                .into())
            }
            _ => panic!("Unexpected non local exit status {}", status),
        }
    }

    /// Converts a Rust's `Result` to either a normal value, or a non-local exit in Lisp.
    #[inline]
    pub(crate) unsafe fn maybe_exit(&self, result: Result<Value<'_>>) -> emacs_value {
        match result {
            Ok(v) => v.raw,
            Err(error) => match error.downcast_ref::<ErrorKind>() {
                Some(err) => {
                    // SAFETY: TempValue's raw values remain live for the duration of this error.
                    unsafe { self.handle_known(err) }
                }
                _ => self
                    .signal_internal_message(&symbol::RUST_ERROR, &format!("{}", error))
                    .unwrap_or_else(|_| panic!("Failed to signal {}", error)),
            },
        }
    }

    /// Converts a caught unwinding panic into a non-local exit in Lisp.
    ///
    /// If there was no error, return the raw `emacs_value`.
    #[inline]
    pub(crate) fn handle_panic(&self, result: thread::Result<emacs_value>) -> emacs_value {
        match result {
            Ok(v) => v,
            Err(error) => {
                // TODO: Try to check for some common types to display?
                let mut m: result::Result<String, Box<dyn Any>> = Err(error);
                if let Err(error) = m {
                    m = error.downcast::<String>().map(|v| *v);
                }
                // TODO: Remove this when we remove `unwrap_or_propagate`.
                if let Err(error) = m {
                    m = match error.downcast::<ErrorKind>() {
                        // TODO: Explain safety.
                        Ok(err) => unsafe {
                            return self.handle_known(&err);
                        },
                        Err(error) => Err(error),
                    }
                }
                if let Err(error) = m {
                    m = Ok(format!("{:#?}", error));
                }
                self.signal_internal_message(&symbol::RUST_PANIC, &m.expect("Logic error"))
                    .expect("Fail to signal panic")
            }
        }
    }

    pub(crate) fn define_core_errors(&self) -> Result<()> {
        // FIX: Make panics louder than errors, by somehow make sure that 'rust-panic is
        // not a sub-type of 'error.
        self.define_error(&symbol::RUST_PANIC, "Rust panic", (&symbol::ERROR,))?;
        self.define_error(&symbol::RUST_ERROR, "Rust error", (&symbol::ERROR,))?;
        self.define_error(
            &symbol::RUST_WRONG_TYPE_USER_PTR,
            "Wrong type user-ptr",
            (&symbol::RUST_ERROR, self.intern("wrong-type-argument")?),
        )?;
        Ok(())
    }

    unsafe fn handle_known(&self, err: &ErrorKind) -> emacs_value {
        match err {
            ErrorKind::Signal { symbol, data } => {
                // SAFETY: TempValue's raw values remain live for the duration of this error.
                unsafe { self.non_local_exit_signal(symbol.raw, data.raw) }
            }
            ErrorKind::Throw { tag, value } => {
                // SAFETY: TempValue's raw values remain live for the duration of this error.
                unsafe { self.non_local_exit_throw(tag.raw, value.raw) }
            }
            ErrorKind::WrongTypeUserPtr { .. } => self
                .signal_internal_message(&symbol::RUST_WRONG_TYPE_USER_PTR, &format!("{}", err))
                .unwrap_or_else(|_| panic!("Failed to signal {}", err)),
        }
    }

    fn signal_internal_message(
        &self,
        symbol: &LazyGlobalRef<Symbol>,
        message: &str,
    ) -> Result<emacs_value> {
        let message = message.into_lisp(self)?;
        let data = self.list([message])?;
        self.signal_internal(symbol, data)
    }

    fn signal_internal(&self, symbol: &LazyGlobalRef<Symbol>, data: Value) -> Result<emacs_value> {
        unsafe { Ok(self.non_local_exit_signal(symbol.try_bind(self)?.raw, data.raw)) }
    }

    /// Defines a new Lisp error signal. This is the equivalent of the Lisp function's [`define-error`].
    ///
    /// The error name can be either a string, a [`Value`], or a [`GlobalRef`].
    ///
    /// [`define-error`]: https://www.gnu.org/software/emacs/manual/html_node/elisp/Error-Symbols.html
    pub fn define_error<'e, N, P>(&'e self, name: N, message: &str, parents: P) -> Result<Value<'e>>
    where
        N: IntoLispSymbol<'e>,
        P: IntoLispArgs<'e>,
    {
        self.call("define-error", (name.into_lisp_symbol(self)?, message, self.list(parents)?))
    }

    /// Signals a Lisp error. This is the equivalent of the Lisp function's [`signal`].
    ///
    /// [`signal`]: https://www.gnu.org/software/emacs/manual/html_node/elisp/Signaling-Errors.html#index-signal
    pub fn signal<'e, S, D, T>(&'e self, symbol: S, data: D) -> Result<T>
    where
        S: IntoLispSymbol<'e>,
        D: IntoLispArgs<'e>,
    {
        let symbol = TempValue { raw: symbol.into_lisp_symbol(self)?.raw };
        let data = TempValue { raw: self.list(data)?.raw };
        Err(ErrorKind::Signal { symbol, data }.into())
    }

    pub(crate) fn non_local_exit_get(
        &self,
        symbol: &mut MaybeUninit<emacs_value>,
        data: &mut MaybeUninit<emacs_value>,
    ) -> emacs_funcall_exit {
        // Safety: The C code writes to these pointers. It doesn't read from them.
        unsafe_raw_call_no_exit!(self, non_local_exit_get, symbol.as_mut_ptr(), data.as_mut_ptr())
    }

    pub(crate) fn non_local_exit_clear(&self) {
        unsafe_raw_call_no_exit!(self, non_local_exit_clear)
    }

    /// # Safety
    ///
    /// The given raw values must still live.
    #[allow(unused_unsafe)]
    pub(crate) unsafe fn non_local_exit_throw(
        &self,
        tag: emacs_value,
        value: emacs_value,
    ) -> emacs_value {
        unsafe_raw_call_no_exit!(self, non_local_exit_throw, tag, value);
        tag
    }

    /// # Safety
    ///
    /// The given raw values must still live.
    #[allow(unused_unsafe)]
    pub(crate) unsafe fn non_local_exit_signal(
        &self,
        symbol: emacs_value,
        data: emacs_value,
    ) -> emacs_value {
        unsafe_raw_call_no_exit!(self, non_local_exit_signal, symbol, data);
        symbol
    }
}

/// Emacs-specific extension methods for the standard library's [`Result`].
///
/// [`Result`]: result::Result
pub trait ResultExt<T, E> {
    /// Converts the error into a Lisp signal if this result is an [`Err`]. The first element of the
    /// associated signal data will be a string formatted with [`Display::fmt`].
    ///
    /// If the result is an [`Ok`], it is returned unchanged.
    fn or_signal<'e, S>(self, env: &'e Env, symbol: S) -> Result<T>
    where
        S: IntoLispSymbol<'e>;
}

impl<T, E: Display> ResultExt<T, E> for result::Result<T, E> {
    fn or_signal<'e, S>(self, env: &'e Env, symbol: S) -> Result<T>
    where
        S: IntoLispSymbol<'e>,
    {
        self.or_else(|err| env.signal(symbol, (format!("{}", err),)))
    }
}
