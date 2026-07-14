# Calling Lisp Functions

Frequently-used Lisp functions are exposed as methods on `env`:

```rust
use rem::IntoLisp;

#[rem::defun]
fn foo(env: &rem::Env) -> rem::Result<()> {
    env.intern("defun")?;

    env.message("Hello")?;

    env.type_of(5_i32.into_lisp(env)?)?;

    env.provide("my-module")?;

    env.list((1, "str", true))?;

    Ok(())
}
```

To call arbitrary Lisp functions, use `env.call(func, args)`.
- `func` can be:
  + A string identifying a named function in Lisp.
  + Any Lisp-callable `Value` (a symbol with a function assigned, a lambda, a subr). This can also be written as `func.call(args)`.
- `args` can be:
  + An array, or a slice of `Value`.
  + A tuple of different types, each satisfying the `IntoLisp` trait.

```rust
use rem::{Env, Result};
#[rem::defun]
fn foo(env: &Env) -> Result<()> {
    // (list "str" 2)
    env.call("list", ("str", 2))?;
    Ok(())
}
```

```rust
#[rem::defun]
fn foo(env: &rem::Env) -> rem::Result<()> {
    let list = env.intern("list")?;
    // (symbol-function 'list)
    let subr = env.call("symbol-function", [list])?;
    // (funcall 'list "str" 2)
    env.call(list, ("str", 2))?;
    // (funcall (symbol-function 'list) "str" 2)
    env.call(subr, ("str", 2))?;
    subr.call(env, ("str", 2))?; // Like the above, but shorter.
    Ok(())
}
```

```rust
// (add-hook 'text-mode-hook 'variable-pitch-mode)
#[rem::defun]
fn foo(env: &rem::Env) -> rem::Result<()> {
    env.call("add-hook", [
        env.intern("text-mode-hook")?,
        env.intern("variable-pitch-mode")?,
    ])?;
    Ok(())
}
```

## Caching Symbols and Functions

Every call to `env.intern` and every symbol-lookup in `env.call("name", ...)` does a hash-table lookup inside Emacs. For hot paths, cache the result with `use_symbols!` or `use_functions!`.

### `use_symbols!`

`use_symbols!` declares `static` variables of type `&OnceGlobalRef` that hold interned symbol values. The variables are initialized once when the module is loaded.

```rust
use rem::{defun, use_symbols, Result, Value};

use_symbols! {
    LEFT => "left",
    RIGHT => "right",
    CENTER => "center",
}

#[defun]
fn classify(env: &rem::Env, pos: Value<'_>) -> Result<String> {
    if pos.eq(env, LEFT.try_bind(env)?) {
        Ok("left".to_owned())
    } else if pos.eq(env, RIGHT.try_bind(env)?) {
        Ok("right".to_owned())
    } else if pos.eq(env, CENTER.try_bind(env)?) {
        Ok("center".to_owned())
    } else {
        Ok("unknown".to_owned())
    }
}
```

The Lisp name for each symbol is derived by replacing `_` with `-`. Use `=> "lisp-name"` to override:

```rust
rem::use_symbols! {
    NIL => "nil",
    T => "t",
    BUFFER_READ_ONLY => "buffer-read-only"
}
```

If the symbol is bound to a function, you can call it via `env.call(symbol_var, args)`. This goes through symbol lookup on each call. Use `use_functions!` to avoid that indirection.

### `use_functions!`

`use_functions!` is like `use_symbols!`, but stores the function object directly (via `indirect-function`). Calls through these variables skip symbol lookup entirely.

```rust
use rem::{defun, use_functions, Env, Result, Value};

use_functions! {
    MESSAGE => "message",
    STRING_TO_NUMBER => "string-to-number",
}

#[defun]
fn greet_parsed(env: &Env, s: String) -> Result<()> {
    let n: i64 = env.call(&STRING_TO_NUMBER, (s,))?.into_rust(env)?;
    env.call(&MESSAGE, (format!("Got {}", n),))?;
    Ok(())
}
```

**Trade-off**: `use_functions!` is faster than `use_symbols!` for repeated calls because it skips symbol lookup. However, if the symbol is later rebound to a different function, the cached reference still points to the original function. Use `use_symbols!` when you need to respect runtime rebinding; use `use_functions!` for built-in and primitive functions where rebinding is not expected.

Both macros can be used only once per Rust `mod`. To cover multiple `mod`s, place one invocation in each.
