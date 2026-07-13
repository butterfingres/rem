//! Testing `PartialEq` for `Value` and `GlobalRef`.

use rem::{defun, use_symbols, Env, GlobalRef, IntoLisp, Result, Value};

// Symbols representing the three directions a node can appear in.
use_symbols! {
    LEFT => "left",
    RIGHT => "right",
    CENTER => "center",
}

/// Return t if A and B are `eq` (using Rust's `==` operator on `Value`).
#[defun(name = ValueEq)]
fn value_eq<'e>(env: &'e Env, a: Value<'e>, b: Value<'e>) -> Result<Value<'e>> {
    (a.eq(env, b)).into_lisp(env)
}

/// Return t if GLOBAL-REF is `eq` to VALUE (using Rust's `==` on `GlobalRef`).
#[defun(name = GlobalRefEq)]
fn global_ref_eq<'e>(env: &'e Env, global: GlobalRef, value: Value<'e>) -> Result<Value<'e>> {
    (global.eq(env, value)).into_lisp(env)
}

/// Return a newly allocated string with the given content, for testing that equal
/// strings are not `eq`.
#[defun(name = NewString)]
fn new_string(env: &Env, s: String) -> Result<Value<'_>> {
    s.into_lisp(env)
}

/// Classify a node's POSITION symbol as one of "left", "right", or "center".
/// Returns an error string for unrecognized symbols.
///
/// This illustrates the idiomatic use of `PartialEq` on `Value`: comparing an
/// incoming argument against `use_symbols!`-imported `OnceGlobalRef` globals
/// avoiding repeated `intern` calls on the hot path.
#[defun(name = ClassifyPosition)]
fn classify_position(env: &Env, position: Value<'_>) -> Result<String> {
    if LEFT.eq(env, position) {
        Ok("left".to_owned())
    } else if RIGHT.eq(env, position) {
        Ok("right".to_owned())
    } else if CENTER.eq(env, position) {
        Ok("center".to_owned())
    } else {
        Ok("unknown".to_owned())
    }
}

pub fn init(env: &Env) -> Result<()> {
    env.lambda(&ValueEq, None)?.fset("t/eq:value-eq")?;
    env.lambda(&GlobalRefEq, None)?.fset("t/eq:global-ref-eq")?;
    env.lambda(&NewString, None)?.fset("t/eq:new-string")?;
    env.lambda(&ClassifyPosition, None)?.fset("t/eq:classify-position")?;

    Ok(())
}
