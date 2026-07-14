# Type Conversions

The type `Value` represents Lisp values:
- They can be copied around, but cannot outlive the `Env` they come from.
- They are "proxy values": only useful when converted to Rust values, or used as arguments when calling Lisp functions.

## Converting a Lisp `Value` to Rust

This is enabled for types that implement `FromLisp`. Most built-in types are supported. Note that conversion may fail, so the return type is `Result<T>`.

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

It's better to declare input types for `#[defun]` than calling `.into_rust()`, unless delayed conversion is needed.

## Converting a Rust Value to Lisp

This is enabled for types that implement `IntoLisp`. Most built-in types are supported. Note that conversion may fail, so the return type is `Result<Value<'_>>`.

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

It's better to declare return type for `#[defun]` than calling `.into_lisp(env)`, whenever possible.

## Integers

Integer conversion is lossless by default, which means that a module will signal an "out of range" `rust-error` in cases such as:
- A `#[defun]` expecting `u8` gets passed `-1`.
- A `#[defun]` returning `u64` returns a value larger than `i64::max_value()`.

To disable this behavior, use the `lossy-integer-conversion` feature:

```toml
[dependencies.emacs]
features = ["lossy-integer-conversion"]
```

Support for Rust's `NonZero` integer types is disabled by default. To enable it, use the `nonzero-integer-conversion` feature:
```toml
[dependencies.emacs]
features = ["nonzero-integer-conversion"]
```

## Strings

By default, no utf-8 validation is done when converting Lisp strings into Rust strings, because the string data returned by Emacs is guaranteed to be valid utf-8 sequence. If you think you've otherwise encountered an Emacs bug, utf-8 validation can be enabled through a feature:

```toml
[dependencies.emacs]
features = ["utf-8-validation"]
```

## Equality

`Value` implements `PartialEq`, which maps to Lisp's `eq` (identity/pointer equality, not `equal`).

```rust
use rem::{defun, Env, IntoLisp, Result};

#[defun]
fn foo(env: &Env) -> Result<()> {
    // Two references to the same interned symbol are eq.
    let a = env.intern("hello")?;
    let b = env.intern("hello")?;
    assert!(a.eq(env, b));

    // Two separately allocated strings with the same content are not eq.
    let s1 = "hi".into_lisp(env)?;
    let s2 = "hi".into_lisp(env)?;
    assert!(s1.eq(env, s2));

    Ok(())
}
```

`GlobalRef` and `OnceGlobalRef` implement `PartialEq<Value>` (and vice versa), so you can compare a cached global against an incoming argument without rebinding:

```rust
use rem::{use_symbols, Env, Result, Value};

use_symbols! {
    NIL => "nil",
}

fn is_nil(env: &Env, v: Value<'_>) -> Result<bool> {
    Ok(v.eq(env, NIL.try_bind(env)?))
}
```

The old `value.eq(other)` method is deprecated since 0.20.0. Use `==` instead.

## Vectors

Lisp vectors are represented by the type `Vector`, which can be considered a "sub-type" of `Value`.

To construct Lisp vectors, use `env.make_vector` and `env.vector`, which are efficient wrappers of Emacs's built-in subroutines `make-vector` and `vector`.

```rust
use rem::{defun, Env, Result};

fn test_vectors(env: &Env) -> Result<()> {
    env.make_vector(5, ())?;

    env.vector((1, 2, 3))?;

    env.vector((1, "x", true))?;

    Ok(())
}
```
