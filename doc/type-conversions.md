# Type Conversions

The type `Value` represents Lisp values:
- They can be copied around, but cannot outlive the `Env` they come
  from.
- They are "proxy values": only useful when converted to Rust values,
  or used as arguments when calling Lisp functions.

## Converting a Lisp `Value` to Rust

This is enabled for types that implement
[`FromLisp`](crate::FromLisp). Most built-in types are supported. Note
that conversion may fail, so the return type is [`Result<T>`](crate::Result).

```rust
use rem::{Env, Result, Value};

fn identity_i64(env: &Env, value: &Value) -> Result<i64> {
    let i: i64 = value.into_rust(env)?; // error if Lisp value is not an integer
    Ok(i)
}
fn identity_f64(env: &Env, value: &Value) -> Result<f64> {
    let f: f64 = value.into_rust(env)?; // error if Lisp value is nil
    Ok(f)
}

fn identity_string(env: &Env, value: &Value) -> Result<String> {
    let s = value.into_rust::<String>(env)?;
    Ok(s)
}
fn identity_optional_string(env: &Env, value: &Value) -> Result<Option<String>> {
    let s = value.into_rust::<Option<String>>(env)?; // None if Lisp value is nil
    Ok(s)
}
```

It's better to declare input types for [`#[defun]`](crate::defun) than
calling [`.into_rust()`](crate::Value::into_rust), unless delayed
conversion is needed.

## Converting a Rust Value to Lisp

This is enabled for types that implement [`IntoLisp`](crate::IntoLisp). Most built-in
types are supported. Note that conversion may fail, so the return type
is `Result<Value<'_>>`.

```rust
use rem::IntoLisp;

#[rem::defun]
fn foo(env: &rem::Env) -> rem::Result<()> {
    "abc".into_lisp(env)?;
    "a\0bc".into_lisp(env)?; // NulError (Lisp string cannot contain null byte)

    5.into_lisp(env)?;
    65.3.into_lisp(env)?;

    ().into_lisp(env)?; // nil
    true.into_lisp(env)?; // t
    false.into_lisp(env)?; // nil

    Ok(())
}
```

It's better to declare return type for [`#[defun]`](crate::defun) than
calling [`.into_lisp(env)`](crate::IntoLisp::into_lisp), whenever
possible.

## Integers

Integer conversion is lossless by default, which means that a module will signal an "out of range" `rust-error` in cases such as:
- A [`#[defun]`](crate::defun) expecting [`u8`] gets passed `-1`.
- A [`#[defun]`](crate::defun) returning [`u64`] returns a value larger than [u64::MAX].

To disable this behavior, use the `lossy-integer-conversion` feature:

```toml
[dependencies.rem]
features = ["lossy-integer-conversion"]
```

Support for Rust's `NonZero` integer types is disabled by default. To enable it, use the `nonzero-integer-conversion` feature:
```toml
[dependencies.rem]
features = ["nonzero-integer-conversion"]
```

## Strings

By default, no utf-8 validation is done when converting Lisp strings
into Rust strings, because the string data returned by Emacs is
guaranteed to be valid utf-8 sequence. If you think you've otherwise
encountered an Emacs bug, utf-8 validation can be enabled through a
feature:

```toml
[dependencies.emacs]
features = ["utf-8-validation"]
```

## Vectors

Lisp vectors are represented by the type [`Vector`](crate::Vector),
which can be considered a "sub-type" of [`Value`](crate::Value).

To construct Lisp vectors, use
[`Env::make_vector`](crate::Env::make_vector) and
[`env.vector`](crate::Env::vector), which are efficient wrappers of
Emacs's built-in subroutines `make-vector` and `vector`.

```rust
use rem::{defun, Env, Result};

fn test_vectors(env: &Env) -> Result<()> {
    env.make_vector(5, ())?;

    env.vector((1, 2, 3))?;

    env.vector((1, "x", true))?;

    Ok(())
}
```
