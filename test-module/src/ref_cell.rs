//! Testing RefCell embedded in user-ptr.

use rem::{defun, Env, Result, Value};
use std::cell::RefCell;

// TODO: Add tests for Mutex and RwLock, and more tests for RefCell.

/// Wrap the given integer in a RefCell.
#[defun(user_ptr)]
fn wrap(x: i64) -> Result<i64> {
    Ok(x)
}

#[defun]
fn unwrap(env: &Env, r: Value<'_>) -> Result<i64> {
    let r: &RefCell<i64> = r.into_rust(env)?;
    Ok(*r.try_borrow()?)
}

/// Mutably increment the wrapped integer, returning the new value.
#[defun]
fn inc(x: &mut i64) -> Result<i64> {
    *x += 1;
    Ok(*x)
}

/// Unwrap the integer, call the given function while still holding the reference.
#[defun]
#[allow(clippy::trivially_copy_pass_by_ref)] // TODO: Test with sth else not i64.
fn unwrap_and_call(env: &Env, _: &i64, lambda: Value<'_>) -> Result<()> {
    lambda.call(env, [])?;
    Ok(())
}

pub fn init(env: &Env) -> Result<()> {
    env.lambda(&Wrap, None)?.fset("t/ref-cell-wrap")?;
    env.lambda(&Unwrap, None)?.fset("t/ref-cell-unwrap")?;
    env.lambda(&Inc, None)?.fset("t/ref-cell-inc")?;
    env.lambda(&UnwrapAndCall, None)?.fset("t/ref-cell-unwrap-and-call")?;

    Ok(())
}
