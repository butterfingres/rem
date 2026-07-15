# Declaring a Module

Each dynamic module must have an initialization function, marked by
the attribute macro [`#[rem::module]`](crate::module). The function's
type must be `fn(&Env) -> Result<()>`.

In addition, in order to be loadable by Emacs, the module must be
declared GPL-compatible.

```rust
use rem::{Env, Result};

rem::plugin_is_GPL_compatible!();

#[rem::module]
fn init(env: &Env) -> Result<()> {
    // This is run when Emacs loads the module.
    // More concretely, it is run before `(provide 'feature-name)` is (automatically) called.
    Ok(())
}
```

## Options

- `name`: By default, the name of the feature provided by the module
  is the crate's name (with `_` replaced by `-`). There is no need to
  explicitly call `provide` inside the initialization function. This
  option allows the function's name, or a string, to be used instead.

    ```rust
    // Putting `rs` in crate's name is discouraged so we use the function's name
    // instead. The feature will be `rs-module-helper`.
    #[rem::module(name(fn))]
    fn rs_module_helper(_: &rem::Env) -> rem::Result<()> { Ok(()) }
    ```

    ```rust
    #[rem::module(name("foo"))]
    fn init(_: &rem::Env) -> rem::Result<()> { Ok(()) }
    ```
