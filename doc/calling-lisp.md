# Calling Lisp Functions

Frequently-used Lisp functions are exposed as methods on [`Env`](crate::Env):

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

To call arbitrary Lisp functions, use [`env.call(func, args)`](crate::Env::call).
- `func` can be:
  + A string identifying a named function in Lisp.
  + Any Lisp-callable [`Value`](crate::Value) (a symbol with a function
    assigned, a lambda, a subr). This can also be written as
    [`func.call(env, args)`](crate::Value::call).
- `args` can be:
  + An array, or a slice of [`Value`](crate::Value).
  + A tuple of different types, each satisfying the [`IntoLisp`](crate::IntoLisp) trait.

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
#[rem::defun]
fn foo(env: &rem::Env) -> rem::Result<()> {
    // (add-hook 'text-mode-hook 'variable-pitch-mode)
    env.call("add-hook", [
        env.intern("text-mode-hook")?,
        env.intern("variable-pitch-mode")?,
    ])?;
    Ok(())
}
```

## Caching Symbols and Functions

Every call to [`Env::intern`](crate::Env::intern) and every
symbol-lookup in [`Env::call`](crate::Env::call) does a hash-table
lookup inside Emacs. For hot paths, cache the result with
[`use_symbols!`](crate::use_symbols) or
[`use_functions!`](crate::use_functions).

### [`use_symbols!`](crate::use_symbols)

[`use_symbols!`](crate::use_symbols) declares `static` variables of
type [`&LazyGlobalRef`](crate::LazyGlobalRef) that hold interned
symbol values. The variables are initialized once when the module is
loaded.

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

```rust
rem::use_symbols! {
    NIL => "nil",
    T => "t",
    BUFFER_READ_ONLY => "buffer-read-only"
}
```

If the symbol is bound to a function, you can call it via
[`env.call(symbol_var, args)`](crate::Env::call). This goes through
symbol lookup on each call. Use
[`use_functions!`](crate::use_functions) to avoid that indirection.

### [`use_functions!`](crate::use_functions)

[`use_functions!`](crate::use_functions) is like
[`use_symbols!`](crate::use_symbols), but stores the function object
directly (via `indirect-function`). Calls through these variables skip
symbol lookup entirely.

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

**Trade-off**: [`use_functions!`](crate::use_functions) is faster than
  [`use_symbols!`](crate::use_symbols) for repeated calls because it
  skips symbol lookup. However, if the symbol is later rebound to a
  different function, the cached reference still points to the
  original function. Use [`use_symbols!`](crate::use_symbols) when you
  need to respect runtime rebinding; use
  [`use_functions!`](crate::use_functions) for built-in and primitive
  functions where rebinding is not expected.

Both macros can be used only once per Rust `mod`. To cover multiple `mod`s, place one invocation in each.
