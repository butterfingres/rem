# Emacs Module in Rust
[User Guide](./guide/src/SUMMARY.md) | [Change Log](./CHANGELOG.md)

This provides a high-level binding to `emacs-module`, Emacs's support for dynamic modules.

Code for a minimal module looks like this:

```rust
use rem::{defun, Env, Result, Value};

rem::plugin_is_GPL_compatible!();

#[emacs::module(name = "greeting")]
fn init(env: &Env) -> Result<()> {
    env.lambda(&SayHello, None)?.fset("greeting-say-hello")?;
    Ok(())
}

#[defun(name = SayHello)]
fn say_hello(env: &Env, name: String) -> Result<Value<'_>> {
    env.message(&format!("Hello, {}!", name))
}
```

```emacs-lisp
(require 'greeting)
(greeting-say-hello "Emacs")
```

## Development

- Building:
    ```shell
    cargo xtask build
    ```
- Testing:
    ```shell
    cargo xtask test
    ```
- Continuous testing (requires `cargo-watch`):
    ```shell
    cargo xtask test --watch
    ```
