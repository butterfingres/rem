#![allow(mismatched_lifetime_syntaxes)]

use std::{env, panic, sync::LazyLock};

use rem::{defun, CallEnv, Env, IntoLisp, Result, Value};

#[macro_use]
mod macros;

mod test_basics;
mod test_eq;
mod test_error;
mod test_lifetime;
mod test_vector;
mod call;

mod ref_cell;
mod vector;
mod hash_map;

rem::plugin_is_GPL_compatible!();

const MODULE: &str = "t";
static MODULE_PREFIX: LazyLock<String> = LazyLock::new(|| format!("{}/", MODULE));

// TODO: Add more tests for different combinations of module options.
#[rem::module(name(fn), separator = "/")]
fn t(env: &Env) -> Result<()> {
    if let Err(env::VarError::NotPresent) = env::var("RUST_BACKTRACE") {
        // Silence panic logging.
        panic::set_hook(Box::new(|_| {}));
    }

    env.message("Hel\0lo, \0Emacs")?;

    call::init(env)?;
    hash_map::init(env)?;
    ref_cell::init(env)?;
    vector::init(env)?;
    test_basics::init(env)?;
    test_eq::init(env)?;
    test_error::init(env)?;
    test_lifetime::init(env)?;
    test_vector::init(env)?;
    env.lambda(&Inc, None)?.fset(env, "t/inc")?;
    env.lambda(&Identity, Some(c"Return the input (not a copy)."))?.fset(env, "t/identity")?;
    env.lambda(&ToUppercase, None)?.fset(env, "t/to-uppercase")?;
    env.lambda(&WrapString, None)?.fset(env, "t/wrap-string")?;
    env.lambda(&MakeDec, None)?.fset(env, "t/make-dec")?;
    env.lambda(&MakeIncAndPlus, None)?.fset(env, "t/make-inc-and-plus")?;

    Ok(())
}

// -----------------------------------------------------------------------------
// Below are tests for functions declared at root of the crate. Don't move them elsewhere.

// Docstring above, with space.
/// 1+
#[defun(name = Inc)]
fn inc(x: i64) -> Result<i64> {
    Ok(x + 1)
}

// Docstring below, without space.
#[defun(name = Identity)]
fn identity(x: Value) -> Result<Value> {
    Ok(x)
}

#[defun(name = ToUppercase)]
fn to_uppercase(s: String) -> Result<String> {
    Ok(s.to_uppercase())
}

#[allow(dead_code)]
struct StringWrapper {
    pub s: String,
}

custom_types! {
    StringWrapper;
}

#[defun(name = WrapString)]
fn wrap_string(s: String) -> Result<Box<StringWrapper>> {
    Ok(Box::new(StringWrapper { s }))
}

#[defun(name = MakeDec)]
fn make_dec(env: &Env) -> Result<Value<'_>> {
    fn dec(env: &CallEnv) -> Result<Value<'_>> {
        let i: i64 = env.parse_arg(0)?;
        (i - 1).into_lisp(env)
    }
    rem::lambda!(env, dec, 1..1, "decrement")
}

#[defun(name = MakeIncAndPlus)]
fn make_inc_and_plus(env: &Env) -> Result<Value<'_>> {
    fn inc(env: &CallEnv) -> Result<Value<'_>> {
        let i: i64 = env.parse_arg(0)?;
        (i + 1).into_lisp(env)
    }

    fn plus(env: &CallEnv) -> Result<Value<'_>> {
        let x: i64 = env.parse_arg(0)?;
        let y: i64 = env.parse_arg(1)?;
        (x + y).into_lisp(env)
    }

    env.call("cons", &[rem::lambda!(env, inc, 1..1, "increment")?, rem::lambda!(env, plus, 2..2)?])
}
