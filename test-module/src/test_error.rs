//! Testing error reporting and handling.

use std::fs;

use rem::{defun, CallEnv, Env, Result, Value};
use rem::ErrorKind;
use rem::ResultExt;

use super::MODULE_PREFIX;

#[defun(name = LispDivide)]
fn lisp_divide(env: &Env, x: Value<'_>, y: Value<'_>) -> Result<i64> {
    fn inner(env: &Env, x: i64, y: i64) -> Result<Value<'_>> {
        call!(env, "/", x, y)
    }

    fn foo<'e>(env: &'e Env, x: Value<'_>, y: Value<'_>) -> Result<Value<'e>> {
        inner(env, x.into_rust(env)?, y.into_rust(env)?)
    }

    foo(env, x, y)?.into_rust(env)
}

#[defun(name = GetType)]
fn get_type<'e>(env: &'e Env, f: Value<'e>) -> Result<Value<'e>> {
    match f.call(env, []) {
        Err(error) => {
            if let Some(ErrorKind::Signal { symbol, .. }) = error.downcast_ref::<ErrorKind>() {
                unsafe {
                    return Ok(symbol.value(env));
                }
            }
            Err(error)
        }
        v => v,
    }
}

/// Call LAMBDA and return the result. Return the thrown value if EXPECTED-TAG is thrown.
#[defun(name = Catch)]
fn catch<'e>(env: &'e Env, expected_tag: Value<'e>, lambda: Value<'e>) -> Result<Value<'e>> {
    match lambda.call(env, []) {
        Err(error) => {
            if let Some(ErrorKind::Throw { tag, value }) = error.downcast_ref::<ErrorKind>() {
                unsafe {
                    if tag.value(env).eq(env, expected_tag) {
                        return Ok(value.value(env));
                    }
                }
            }
            Err(error)
        }
        v => v,
    }
}

/// Call `apply` on LAMBDA and ARGS, propagating any signaled error.
#[defun(name = Apply)]
fn apply<'e>(env: &'e Env, lambda: Value<'e>, args: Value<'e>) -> Result<Value<'e>> {
    env.call("apply", (lambda, args))
}

#[defun(name = ReadFile)]
fn read_file<'e>(env: &Env, path: String) -> Result<String> {
    fs::read_to_string(path).or_signal(env, &EMRS_FILE_ERROR)
}

#[defun(name = Panic)]
fn panic(message: String) -> Result<()> {
    panic!("{}", message)
}

#[defun(name = Signal)]
fn signal(env: &Env, symbol: Value, message: String) -> Result<()> {
    env.signal(symbol, (message,))
}

fn parse_arg(env: &CallEnv) -> Result<String> {
    let i: i64 = env.parse_arg(0)?;
    let s: String = env.parse_arg(i as usize)?;
    Ok(s)
}

rem::use_symbols! {
    EMRS_FILE_ERROR => "emrs-file-error",
    EMACS_MODULE_RS_TEST_ERROR => "emacs-module-rs-test-error",
    ERROR_DEFINED_WITHOUT_PARENT => "error-defined-without-parent",
    RUST_ERROR => "rust-error",
}

pub fn init(env: &Env) -> Result<()> {
    env.define_error(&EMRS_FILE_ERROR, "File error", [])?;
    env.define_error(&EMACS_MODULE_RS_TEST_ERROR, "Hello", [RUST_ERROR.try_bind(env)?])?;
    env.define_error(&ERROR_DEFINED_WITHOUT_PARENT, "Error", [])?;

    rem::__export_functions! {
        env, format!("{}error:", *MODULE_PREFIX), {
            "parse-arg"   => (parse_arg   , 2..5),
        }
    }

    #[defun(name = SignalCustom)]
    fn signal_custom(env: &Env) -> Result<()> {
        env.signal(&EMACS_MODULE_RS_TEST_ERROR, [])
    }

    env.lambda(&LispDivide, None)?.fset(env, "t/error:lisp-divide")?;
    env.lambda(&GetType, None)?.fset(env, "t/error:get-type")?;
    env.lambda(&Catch, None)?.fset(env, "t/error:catch")?;
    env.lambda(&Apply, None)?.fset(env, "t/error:apply")?;
    env.lambda(&ReadFile, None)?.fset(env, "t/read-file")?;
    env.lambda(&Panic, None)?.fset(env, "t/error:panic")?;
    env.lambda(&Signal, None)?.fset(env, "t/error:signal")?;
    env.lambda(&SignalCustom, None)?.fset(env, "t/error:signal-custom")?;

    Ok(())
}
