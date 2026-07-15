# Hello, Emacs!

Create a new project:

```bash
cargo new greeting
cd greeting
```

Modify `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
rem = { version = "1.0.0", git = "https://github.com/butterfingres/rem.git" }
```

Write code in `src/lib.rs`:

```rust
use rem::{defun, Env, Result, Value};

// Emacs won't load the module without this.
rem::plugin_is_GPL_compatible!();

// Register the initialization hook that Emacs will call when it loads the module.
#[rem::module]
fn init(env: &Env) -> Result<Value<'_>> {
    env.lambda(&SayHello, None)?.fset("greeting-say-hello")?;
    env.message("Done loading!")
}

// Define a function callable by Lisp code.
#[defun]
fn say_hello(env: &Env, name: String) -> Result<Value<'_>> {
    env.message(&format!("Hello, {}!", name))
}
```

Build the module and create a symlink with `.so` extension so that Emacs can recognize it:

```bash
cargo build
cd target/debug

# If you are on Linux
ln -s libgreeting.so greeting.so

# If you are on macOS
ln -s libgreeting.dylib greeting.so
```

Add `target/debug` to your Emacs's `load-path`, then load the module:
```lisp
(add-to-list 'load-path "/path/to/target/debug")
(require 'greeting)
(greeting-say-hello "Emacs")
```

The minibuffer should display the message `Hello, Emacs!`.
