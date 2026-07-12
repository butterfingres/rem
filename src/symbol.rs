use crate::{
    Env, Result, Value,
    global::{GlobalRef, LazyGlobalRef, Symbol},
};

/// Defines static [`&OnceGlobalRef`] variables that point to corresponding Lisp symbols.
///
/// This macro accepts a space-separated list of identifiers, and determine the Lisp symbol names by
/// replacing underscores with hyphens, or by explicit mappings in the form of `=> "symbol-name"`.
///
/// It can be used only once per Rust `mod`.
///
/// If a symbol is bound to a function, the variable can be used to call the function via
/// [Env::call]. Unlike with [`use_functions!`], this goes through the indirection of symbol lookup.
///
/// [`&OnceGlobalRef`]: OnceGlobalRef
/// [`use_functions!`]: crate::use_functions
#[macro_export]
macro_rules! use_symbols {
    ($($ident:ident => $symbol:expr),* $(,)?) => {
        $(pub static $ident: $crate::LazyGlobalRef<$crate::Symbol> = $crate::LazyGlobalRef::new($crate::Symbol::new($symbol));)*
    };
}

use_symbols! {
    NIL => "nil",
    T => "t",
    ERROR => "error",
    RUST_ERROR => "rust-error",
    RUST_PANIC => "rust-panic",
    RUST_WRONG_TYPE_USER_PTR => "rust-wrong-type-user-ptr",
    WRONG_NUMBER_OF_ARGUMENTS => "wrong-number-of-arguments",
}

pub trait IntoLispSymbol<'e> {
    fn into_lisp_symbol(self, env: &'e Env) -> Result<Value<'e>>;
}

impl<'e> IntoLispSymbol<'e> for Value<'e> {
    #[inline(always)]
    fn into_lisp_symbol(self, _: &'e Env) -> Result<Value<'e>> {
        Ok(self)
    }
}

impl<'e, T: AsRef<str>> IntoLispSymbol<'e> for T {
    #[inline(always)]
    fn into_lisp_symbol(self, env: &'e Env) -> Result<Value<'e>> {
        env.intern(self.as_ref())
    }
}

impl<'e> IntoLispSymbol<'e> for &'e GlobalRef {
    #[inline(always)]
    fn into_lisp_symbol(self, env: &'e Env) -> Result<Value<'e>> {
        self.bind(env).into_lisp_symbol(env)
    }
}
impl<'e> IntoLispSymbol<'e> for &'e LazyGlobalRef<Symbol<'_>> {
    #[inline(always)]
    fn into_lisp_symbol(self, env: &'e Env) -> Result<Value<'e>> {
        self.try_bind(env)?.into_lisp_symbol(env)
    }
}
