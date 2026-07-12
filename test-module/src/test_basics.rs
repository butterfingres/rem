use rem::{defun, CallEnv, Env, IntoLisp, Result, Value};
use rem::func::Manage;

use super::MODULE_PREFIX;

fn using_fset(env: &Env) -> Result<()> {
    make_prefix!(prefix, *MODULE_PREFIX);

    fn sum_and_diff(env: &CallEnv) -> Result<Value<'_>> {
        let x: i64 = env.parse_arg(0)?;
        let y: i64 = env.parse_arg(1)?;
        env.list(&[(x + y).into_lisp(env)?, (x - y).into_lisp(env)?])
    }

    env.fset(prefix!("sum-and-diff"), rem::lambda!(env, sum_and_diff, 2..2)?)?;

    Ok(())
}

#[defun(name = ToLowercaseOrNil)]
fn to_lowercase_or_nil(env: &Env, input: Option<String>) -> Result<Value<'_>> {
    let output = input.map(|s| s.to_lowercase());
    // This tests IntoLisp for Option<&str>.
    output.as_ref().into_lisp(env)
}

// Test that raw identifiers are handled correctly. Note that it must be a reserved keyword,
// otherwise syn parses it into a non-raw identifier.
#[defun(name = Match)]
fn r#match() -> Result<()> {
    Ok(())
}

#[defun(name = IdentityI8)]
fn identity_i8(i: i8) -> Result<i8> {
    Ok(i)
}

#[defun(name = IdentityU8)]
fn identity_u8(i: u8) -> Result<u8> {
    Ok(i)
}

#[defun(name = U64Overflow)]
fn u64_overflow() -> Result<u64> {
    Ok(u64::MAX)
}

#[defun(name = IgnoreArgs)]
fn ignore_args(_: &Env, _: u8, _: u16) -> Result<()> {
    Ok(())
}

#[defun(name = CopyStringContents)]
fn copy_string_contents(env: &Env, v: Value, size: usize) -> Result<String> {
    let mut buffer = vec![0u8; size];
    let s = v.copy_string_contents(env, &mut buffer)?;
    Ok(String::from_utf8_lossy(s).to_string())
}

pub fn init(env: &Env) -> Result<()> {
    using_fset(env)?;
    env.lambda(&ToLowercaseOrNil, None)?.fset(env, "t/to-lowercase-or-nil")?;
    env.lambda(&Match, None)?.fset(env, "t/match")?;
    env.lambda(&IdentityI8, None)?.fset(env, "t/identity-i8")?;
    env.lambda(&IdentityU8, None)?.fset(env, "t/identity-u8")?;
    env.lambda(&U64Overflow, None)?.fset(env, "t/u64-overflow")?;
    env.lambda(&IgnoreArgs, None)?.fset(env, "t/ignore-args")?;
    env.lambda(&CopyStringContents, None)?.fset(env, "t/copy-string-contents")?;

    fn sum(env: &CallEnv) -> Result<i64> {
        let x: i64 = env.parse_arg(0)?;
        let y: i64 = env.parse_arg(1)?;
        Ok(x + y)
    }

    rem::__export_functions! {
        env, *MODULE_PREFIX, {
            "sum" => (sum, 2..2),
        }
    }

    Ok(())
}
