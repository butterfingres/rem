//! Experimental utility macros that make writing a module easier. If they prove to be useful, we
//! will move them into the lib after stabilizing them.

macro_rules! custom_types {
    ($($name:ident;)*) => {$(
        impl ::rem::Transfer for $name {}
    )*};
}

macro_rules! call {
    ($env:ident, $name:expr $(, $arg:expr)*) => {{
        use rem::IntoLisp;
        let args = &[$($arg.into_lisp($env)?,)*];
        $env.call($name, args)
    }}
}
