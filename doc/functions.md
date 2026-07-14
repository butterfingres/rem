# Writing Functions

You can use the attribute macro [`#[defun]`](crate::defun) to create
Rust functions which are accessible to the Lisp runtime, so that Lisp
code can call them. The exporting process should be done manually at
your [module function](crate::module) with
[`Env::lambda`](crate::env::Lambda) and
[`Lambda::fset`](crate::Lambda::fset).

## Input Parameters

Each parameter must be one of the following:
- An owned value of a type that implements `FromLisp`. This is for
  simple data types that have an equivalent in Lisp.
    ```rust
    /// This docstring will not appear in Lisp!
    #[rem::defun]
    fn inc(x: i64) -> rem::Result<i64> {
        Ok(x + 1)
    }
    ```
- A shared/mutable reference. This gives access to data structures
  that other module functions have created and embedded in the Lisp
  runtime (through `user-ptr` objects).
    ```rust
    #[rem::defun]
    fn vec_pop(vec: &mut Vec<u8>) -> rem::Result<()> {
        vec.pop();
        Ok(())
    }
    ```
- A Lisp `Value`, or one of its "sub-types" (e.g. `Vector`). This
  allows holding off the conversion to Rust data structures until
  necessary, or working with values that don't have a meaningful
  representation in Rust, like Lisp lambdas.
  ```rust
  use rem::{defun, Env, Result, Value};

  fn some_hidden_native_logic() -> bool { unimplemented!() }

  #[rem::defun]
  fn maybe_call(env: &Env, lambda: Value) -> Result<()> {
      if some_hidden_native_logic() {
          lambda.call(env, [])?;
      }
      Ok(())
  }
  ```
- An `&Env`. This enables interaction with the Lisp runtime. It does
  not appear in the function's Lisp signature. This is unnecessary if
  there is already another parameter with type `Value`, which allows
  accessing the runtime through `Value.env`.
    ```rust
    use rem::{defun, Env, Result, Value};

    // Note that the function takes an owned `String`, not a reference, which would
    // have been understood as a `user-ptr` object containing a Rust string.
    #[defun]
    fn hello(env: &Env, name: String) -> Result<Value<'_>> {
        env.message(format!("Hello, {}!", name))
    }
    ```

## Return Value

The return type must be `Result<T>`, where `T` is one of the following:
- A type that implements `IntoLisp`. This is for simple data types
  that have an equivalent in Lisp.
  ```rust
  # use rem::{defun, Result};
  #[defun]
  fn dot_git_path(_path: String) -> Result<Option<String>> {
      unimplemented!()
  }
  ```
- An arbitrary type. This allows embedding a native data structure in
  a `user-ptr` object, for read-write use cases. It requires
  `user_ptr` option to be specified. If the data is to be shared with
  background Rust threads, `user_ptr(rwlock)` or `user_ptr(mutex)`
  must be used instead.
  ```rust
  # use rem::{defun, Result};
  struct Foo;
  #[defun(user_ptr)]
  fn make_foo() -> Result<Foo> {
      Ok(Foo)
  }
  ```
- A type that implements `Transfer`. This allows embedding a native
  data structure in a `user-ptr` object, for read-only use cases. It
  requires `user_ptr(direct)` option to be specified.
- `Value`, or one of its "sub-types" (e.g. `Vector`). This is mostly useful for returning an input parameter unchanged.

See [Custom Types](./custom-types.md) for more details on embedding
Rust data structures in Lisp's `user-ptr` objects.

## Naming

By default, the structure that is created by the
[`defun`](crate::defun) attribute is named the name of the function
with underscores (`_`) removed and their next letter capitalized. If
you would like to change this, you can pass in the `name` attribute
argument to change the name of the struct.

Examples:

```rust
use rem::Result;

// Assuming crate's name is `native_parallelism`.
#[rem::module]
fn init(env: &rem::Env) -> Result<()> {
    env.lambda(&shared_state::thread::MakeThread, None)?.fset("native-parallelism/make-thread")?;
    env.lambda(&shared_state::process::Launch, None)?.fset("native-parallelism/shared-state-process-launch")?;
    env.lambda(&shared_state::process::Pool, None)?.fset("native-parallelism/process:pool")?;
    Ok(())
}

mod shared_state {
    pub mod thread {
        use rem::{defun, Result, Value};
        // Ignore the nested mod's.
        // (native-parallelism/make-thread "name")
        #[defun(name = MakeThread)]
        fn make<'e>(name: String) -> Result<Value<'e>> {
            unimplemented!()
        }
    }

    pub mod process {
        use rem::{defun, Result, Value};
        // (native-parallelism/shared-state-process-launch "bckgrnd")
        #[defun]
        fn launch<'e>(name: String) -> Result<Value<'e>> {
            unimplemented!()
        }

        // Specify a name explicitly, since Rust identifier cannot contain `:`.
        // (native-parallelism/process:pool "http-client" 2 8)
        #[defun]
        fn pool<'e>(name: String, min: i64, max: i64) -> Result<Value<'e>> {
            unimplemented!()
        }
    }
}
```

## Registration

The [`defun`](crate::defun) attribute macro only creates a struct that
implements the required traits for [`Env::lambda`](crate::Env::lambda)
so you must register it to the lisp runtime manually.

```rust
use rem::Result;

#[rem::defun]
fn one_plus(x: i32) -> Result<i32> {
    Ok(x + 1)
}

#[rem::module]
fn init(env: &rem::Env) -> Result<()> {
    env.lambda(&OnePlus, None)?.fset("my-1+")?;
    Ok(())
}
```

## Documentation

Documentation must be passed as a [`CStr`](std::ffi::CStr) when
declaring the function.

```rust
use rem::{Env, Result};

#[rem::defun]
fn add(x: usize, y: usize) -> Result<usize> {
    Ok(x + y)
}

#[rem::module]
fn init(env: &Env) -> Result<()> {
    env.lambda(&Add, Some(c"Add 2 numbers.

(fn X Y)"))?.fset("my-add");
    Ok(())
}
```
