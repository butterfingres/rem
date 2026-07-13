#![allow(mismatched_lifetime_syntaxes)]

use std::{env, panic, sync::LazyLock};

use rem::{defun, Env, Lambda, IntoLisp, Result, Value};

#[macro_use]
mod macros;

mod basics;
mod call;
mod eq;
mod error;
mod hash_map;
mod lifetime;
mod ref_cell;
mod vector;
mod vector_2d;

rem::plugin_is_GPL_compatible!();

const MODULE: &str = "t";
static MODULE_PREFIX: LazyLock<String> = LazyLock::new(|| format!("{}/", MODULE));

// TODO: Add more tests for different combinations of module options.
#[rem::module]
fn init(env: &Env) -> Result<()> {
    if let Err(env::VarError::NotPresent) = env::var("RUST_BACKTRACE") {
        // Silence panic logging.
        panic::set_hook(Box::new(|_| {}));
    }

    env.message("Hel\0lo, \0Emacs")?;

    basics::init(env)?;
    call::init(env)?;
    eq::init(env)?;
    error::init(env)?;
    hash_map::init(env)?;
    lifetime::init(env)?;
    ref_cell::init(env)?;
    vector::init(env)?;
    vector_2d::init(env)?;
    env.lambda(&Inc, Some(c"1+"))?.fset("t/inc")?;
    env.lambda(&Identity, Some(c"Return the input (not a copy)."))?.fset("t/identity")?;
    env.lambda(&ToUppercase, None)?.fset("t/to-uppercase")?;
    env.lambda(&WrapString, None)?.fset("t/wrap-string")?;
    env.lambda(&MakeDec, None)?.fset("t/make-dec")?;
    env.lambda(&MakeIncAndPlus, None)?.fset("t/make-inc-and-plus")?;

    Ok(())
}

// -----------------------------------------------------------------------------
// Below are tests for functions declared at root of the crate. Don't move them elsewhere.

// Docstring above, with space.
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
fn make_dec(env: &Env) -> Result<Lambda<'_>> {
    #[defun(name = Dec)]
    fn dec(i: i64) -> Result<i64> {
        Ok(i - 1)
    }
    env.lambda(&Dec, Some(c"decrement"))
}

#[defun(name = MakeIncAndPlus)]
fn make_inc_and_plus(env: &Env) -> Result<Value<'_>> {
    #[defun(name = Inc)]
    fn inc(i: i64) -> Result<i64> {
        Ok(i + 1)
    }

    #[defun(name = Plus)]
    fn plus(x: i64, y: i64) -> Result<i64> {
        Ok(x + y)
    }

    env.call("cons", (env.lambda(&Inc, Some(c"increment"))?, env.lambda(&Plus, Some(c""))?))
}
