# Error Handling and Signaling

Emacs Lisp's [error handling mechanism](https://www.gnu.org/software/emacs/manual/html_node/elisp/Handling-Errors.html)
 uses [non-local exits](https://www.gnu.org/software/emacs/manual/html_node/elisp/Nonlocal-Exits.html).

Rust uses the [`Result`] enum. `rem` converts between the 2 at the
Rust-Lisp boundaries (more precisely, Rust-C).

The chosen error type is the [`Error`](std::error::Error) trait:

```rust
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
```

## Handling Lisp Errors in Rust

When calling a Lisp function, it's usually a good idea to propagate
signaled errors with the `?` operator, letting higher level (Lisp)
code handle them. If you want to handle a specific error, you can use
`error.downcast_ref`:

```rust
# use rem::{Env, ErrorKind, Result};
# #[rem::defun]
# fn foo(env: &Env) -> Result<()> {
#     let some_text = "";
match env.call("insert", (some_text,)) {
    Err(error) => {
        // Handle `buffer-read-only` error.
        if let Some(ErrorKind::Signal { symbol, .. }) = error.downcast_ref::<ErrorKind>() {
            let buffer_read_only = env.intern("buffer-read-only")?;
            // `symbol` is a `TempValue` that must be converted to `Value`.
            let symbol = unsafe { symbol.value(env) };
            if env.eq(symbol, buffer_read_only) {
                env.message("This buffer is not writable!")?;
                return Ok(())
            }
        }
        // Propagate other errors.
        Err(error)
    },
    _ => Ok(()),
}
# }
```

Note the use of `unsafe` to extract the error symbol as a
[`Value`](crate::Value). The reason is that,
[`ErrorKind::Signal`](crate::ErrorKind) is marked `Send + Sync`, for
compatibility with [`std`], while `Value` is lifetime-bound by
`env`. The `unsafe` contract here requires the error being handled
(and its `TempValue`) to come from this `env`, not from another
thread, or from a global/thread-local storage.

### Catching Values Thrown by Lisp

This is similar to handling Lisp errors. The only difference is
[`ErrorKind::Throw`](crate::ErrorKind::Throw) being used instead of
[`ErrorKind::Signal`](crate::ErrorKind::Throw).

## Signaling Lisp Errors from Rust

The function [`Env::signal`](crate::Env::signal) allows signaling a
Lisp error from Rust code. The error symbol must have been defined,
e.g. by the macro [`use_symbols!`](crate::use_symbols) and defined by
[`Env::define_error`](crate::Env::define_error):

```rust
use rem::{defun, Env, Result};

rem::use_symbols! {
    MY_CUSTOM_ERROR => "my-custom-error",
}

#[defun]
fn signal_if_negative(env: &Env, x: i16) -> Result<()> {
    if (x < 0) {
        return env.signal(&MY_CUSTOM_ERROR, ("associated", "DATA", 7))
    }
    Ok(())
}

#[rem::module]
fn init(env: &Env) -> Result<()> {
    env.define_error(&MY_CUSTOM_ERROR, "This number should not be negative", (env.intern("arith-error")?, env.intern("range-error")?))?;
    env.lambda(&SignalIfNegative, None)?.fset("signal-if-negative")?;
    Ok(())
}
```

## Handling Rust Errors in Lisp

In addition to [standard errors](https://www.gnu.org/software/emacs/manual/html_node/elisp/Standard-Errors.html),
 Rust module functions can signal Rust-specific errors, which can also
 be handled by `condition-case`:

- `rust-error`: The message is `Rust error`. This covers all generic
  Rust-originated errors.
- `rust-wrong-type-user-ptr`: The message is `Wrong type
  user-ptr`. This happens when Rust code is passed a `user-ptr` of a
  type it's not expecting. It is a sub-type of `rust-error`.

  ```rust
  use std::{cell::RefCell, collections::HashMap};
  fn get_hash_map(env: &rem::Env, value: &rem::Value) -> rem::Result<()> {
      // May signal if `value` holds a different type of hash map,
      // or is a `user-ptr` defined in a non-Rust module.
      let _r: &RefCell<HashMap<String, String>> = value.into_rust(env)?;
      Ok(())
  }
  ```

### Panics

Unwinding from Rust into C is undefined behavior. `rem` prevents that
by using [`catch_unwind`](std::panic::catch_unwind) at the Rust-to-C
boundary to convert a panic into a Lisp's signal/throw of the
appropriate type:

- Normally the panic is converted into a Lisp's error signal of the
  type `rust-panic`. Note that it is **not a sub-type** of
  `rust-error`.
- If the panic value is an [`ErrorKind`](crate::ErrorKind), it is
  converted to the corresponding signal/throw, as if a [`Result`] was
  returned. This allows propagating Lisp's non-local exits through
  contexts where [`Result`] is not appropriate, e.g. callbacks whose
  types are dictated by 3rd-party libraries, such as `tree-sitter`.
