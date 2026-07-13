#![expect(private_bounds, reason = "the private bounds serve to seal the trait")]

use std::sync::OnceLock;

use emacs_module::emacs_value;

use super::*;

/// A "global reference" that can live outside the scope of an [`Env`]. This is useful for sharing
/// an otherwise short-lived Lisp [`Value`] across multiple invocations of Rust functions defined
/// with [`defun`]. Examples include efficient access to interned symbols or Lisp functions, and
/// Rust-based multi-threading.
///
/// # Implementation
///
/// Cloning this struct requires an [`Env`], so it doesn't implement [`Clone`].
///
/// [`free_global_ref`] requires an [`Env`]. Therefore, to avoid leaking the underlying [`Value`],
/// [`free`] should be used to free a global reference, instead of [`drop`]. For the use case of
/// accessing interned symbols and Lisp functions, this is a non-issue, as the values are
/// supposed to be "static" anyway.
///
/// The above is a shortcoming in the design of emacs-module. There are 2 possible ways to fix it:
/// - Make [`free_global_ref`] work without an env, like Erlang's `enif_release_resource`.
/// - Allow `user_ptr`'s finalizer to access the env, to properly free associated global refs.
///
/// [`Env`]: struct.Env.html
/// [`Value`]: struct.Value.html
/// [`defun`]: attr.defun.html
/// [`Clone`]: https://doc.rust-lang.org/std/clone/trait.Clone.html
/// [`free_global_ref`]: https://www.gnu.org/software/emacs/manual/html_node/elisp/Module-Values.html
/// [`free`]: #method.free
/// [`drop`]: https://doc.rust-lang.org/std/mem/fn.drop.html
#[derive(Debug)]
#[repr(transparent)]
pub struct GlobalRef {
    raw: emacs_value,
}

impl GlobalRef {
    /// Creates a new global reference for the given [`Value`].
    ///
    /// [`Value`]: struct.Value.html
    pub fn new<'e>(env: &'e Env, value: Value<'e>) -> Self {
        // TODO: Check whether this really is `no_exit`.
        let raw = unsafe_raw_call_no_exit!(env, make_global_ref, value.raw);
        // NOTE: raw != value.raw
        Self { raw }
    }

    // For testing.
    pub(crate) unsafe fn from_raw(raw: emacs_value) -> Self {
        Self { raw }
    }

    /// Frees this global reference.
    pub fn free(self, env: &Env) -> Result<()> {
        // Safety: We assume user code doesn't directly call C function `free_global_ref`.
        unsafe_raw_call!(env, free_global_ref, self.raw)?;
        Ok(())
    }

    /// Returns the underlying [`Value`], scoping its lifetime to the given [`Env`].
    ///
    /// [`Env`]: struct.Env.html
    /// [`Value`]: struct.Value.html
    #[inline]
    pub fn bind<'e, 'g: 'e>(&'g self, env: &'e Env) -> Value<'e> {
        // Safety: This global ref keeps the underlying Lisp object alive.
        unsafe { Value::new(self.raw, env) }
    }

    /// Returns a copy of this global reference.
    pub fn clone(&self, env: &Env) -> Self {
        self.bind(env).make_global_ref(env)
    }

    pub fn eq<'e>(&self, env: &'e Env, r: Value<'e>) -> bool {
        self.bind(env).eq(env, r)
    }
}

// Safety: Doing anything useful with a GlobalRef requires an &Env, which means holding the GIL.
unsafe impl Send for GlobalRef {}
unsafe impl Sync for GlobalRef {}

impl<'e> FromLisp<'e> for GlobalRef {
    #[inline(always)]
    fn from_lisp(value: Value<'e>, env: &'e Env) -> Result<Self> {
        Ok(Self::new(env, value))
    }
}

impl<'e> IntoLisp<'e> for &'e GlobalRef {
    #[inline(always)]
    fn into_lisp(self, env: &'e Env) -> Result<Value<'e>> {
        Ok(self.bind(env))
    }
}

impl<'e> Value<'e> {
    /// Creates a new [`GlobalRef`] for this value.
    ///
    /// [`GlobalRef`]: struct.GlobalRef.html
    #[inline(always)]
    pub fn make_global_ref(self, env: &'e Env) -> GlobalRef {
        GlobalRef::new(env, self)
    }
}

trait LazyGlobalRefValue
where
    Self: Copy,
{
    fn initialize<'a>(self, _: &Env, _: &'a LazyGlobalRef<Self>) -> Result<&'a GlobalRef>;
}

#[derive(Clone, Copy, Debug)]
pub struct Symbol<'a>(&'a str);
impl<'a> Symbol<'a> {
    pub const fn new(val: &'a str) -> Self {
        Self(val)
    }
}
impl LazyGlobalRefValue for Symbol<'_> {
    fn initialize<'a>(self, env: &Env, place: &'a LazyGlobalRef<Self>) -> Result<&'a GlobalRef> {
        place.init(env, move |env| env.intern(self.0))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Fn<'a>(&'a str);
impl<'a> Fn<'a> {
    pub const fn new(val: &'a str) -> Self {
        Self(val)
    }
}
impl LazyGlobalRefValue for Fn<'_> {
    fn initialize<'a>(self, env: &Env, place: &'a LazyGlobalRef<Self>) -> Result<&'a GlobalRef> {
        place.init(env, move |env| env.call("indirect-function", (env.intern(self.0)?,)))
    }
}

pub struct LazyGlobalRef<T>
where
    T: LazyGlobalRefValue,
{
    val: T,
    inner: OnceLock<GlobalRef>,
}
impl<T> LazyGlobalRef<T>
where
    T: LazyGlobalRefValue,
{
    pub const fn new(val: T) -> Self {
        Self { val, inner: OnceLock::new() }
    }

    /// Initializes this global reference with the given function.
    fn init<'a, 'e, F: FnOnce(&'e Env) -> Result<Value>>(
        &'a self,
        env: &'e Env,
        f: F,
    ) -> Result<&'a GlobalRef> {
        let g = f(env)?.make_global_ref(env);
        self.inner.set(g).expect("Cannot initialize a global reference more than once");
        Ok(self.inner.get().expect("Failed to get an initialized LazyGlobalRef"))
    }

    pub fn try_bind<'e, 'g: 'e>(&'g self, env: &'e Env) -> Result<Value<'e>> {
        Ok(if let Some(val) = self.inner.get() { val } else { self.val.initialize(env, self)? }
            .bind(env))
    }

    pub fn eq<'e>(&self, env: &'e Env, r: Value<'e>) -> bool {
        self.try_bind(env).map(|l| l.eq(env, r)).unwrap_or_default()
    }
}
impl<'e, T> IntoLisp<'e> for &'e LazyGlobalRef<T>
where
    T: LazyGlobalRefValue,
{
    fn into_lisp(self, env: &'e Env) -> Result<Value<'e>> {
        self.try_bind(env)
    }
}
