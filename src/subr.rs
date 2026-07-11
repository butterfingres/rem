/// Defines static [`&OnceGlobalRef`] variables that point to functions identified by the
/// corresponding symbols.
///
/// This macro accepts a space-separated list of identifiers, and determine the Lisp symbol names by
/// replacing underscores with hyphens, or by explicit mappings in the form of `=> "symbol-name"`.
///
/// It can be used only once per Rust `mod`.
///
/// Unlike with [`use_symbols!`], calling the functions through these variables does not involve the
/// indirection of symbol lookup. That means it is faster, and is not affected by symbol rebinding.
///
/// [`&OnceGlobalRef`]: OnceGlobalRef
/// [`use_symbols!`]: crate::use_symbols
#[macro_export]
macro_rules! use_functions {
    ($($ident:ident => $symbol:expr),* $(,)?) => {
        $(pub static $ident: $crate::LazyGlobalRef<$crate::Fn> = $crate::LazyGlobalRef::new($crate::Fn::new($symbol));)*
    }
}

use_functions! {
    CONS => "cons",
    CAR => "car",
    CDR => "cdr",
    VECTOR => "vector",
    MAKE_VECTOR => "make-vector",
    LIST => "list",
    MESSAGE => "message",
}
