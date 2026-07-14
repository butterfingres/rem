use std::convert::TryInto;

use super::*;
use crate::{subr, call::IntoLispArgs};

/// A type that represents Lisp vectors. This is a wrapper around [`Value`] that provides
/// vector-specific methods.
///
/// Arguments to #[[`defun`]] having this type will be type-checked. If you want to omit, or delay
/// this type checking, use [`Value`] instead.
///
/// ```
/// use rem::{defun, Value, Vector, Result};
///
/// #[defun]
/// fn must_pass_vector(vector: Vector) -> Result<Vector> {
///     Ok(vector)
/// }
///
/// #[defun]
/// fn no_type_check(value: Value) -> Result<Vector> {
///     Ok(Vector::from_value_unchecked(value, 0))
/// }
/// ```
///
/// [`Value`]: struct.Value.html
/// [`defun`]: attr.defun.html
#[derive(Debug, Clone, Copy)]
pub struct Vector<'e> {
    value: Value<'e>,
    len: usize,
}

impl<'e> Vector<'e> {
    #[doc(hidden)]
    #[inline]
    pub fn from_value_unchecked(value: Value<'e>, len: usize) -> Self {
        Self { value, len }
    }

    pub fn get<T: FromLisp<'e>>(&self, env: &'e Env, i: usize) -> Result<T> {
        let v = self.value;
        // Safety:
        // - Same lifetime.
        // - Emacs does bound checking.
        // - Value doesn't need protection because we are done with it while the vector still lives.
        unsafe_raw_call_value_unprotected!(env, vec_get, v.raw, i as isize)?.into_rust(env)
    }

    pub fn set<T: IntoLisp<'e>>(&self, env: &'e Env, i: usize, value: T) -> Result<()> {
        let v = self.value;
        let value = value.into_lisp(env)?;
        // Safety: Same lifetime. Emacs does bound checking.
        unsafe_raw_call!(env, vec_set, v.raw, i as isize, value.raw)
    }

    #[deprecated(since = "0.14.0", note = "Use .len() instead")]
    #[doc(hidden)]
    pub fn size(&self) -> Result<usize> {
        Ok(self.len)
    }

    #[expect(
        clippy::len_without_is_empty,
        reason = "an `is_empty` implementation would not be faster than checking `len() == 0`"
    )]
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    #[inline]
    pub fn value(&self) -> Value<'e> {
        self.value
    }
}

impl<'e> FromLisp<'e> for Vector<'e> {
    fn from_lisp(value: Value<'e>, env: &'e Env) -> Result<Vector<'e>> {
        let len = unsafe_raw_call!(env, vec_size, value.raw)?
            .try_into()
            .expect("Invalid size from Emacs");
        Ok(Vector { value, len })
    }
}

impl<'e> IntoLisp<'e> for Vector<'e> {
    #[inline(always)]
    fn into_lisp(self, _: &'e Env) -> Result<Value<'_>> {
        Ok(self.value)
    }
}

impl Env {
    pub fn make_vector<'e, T: IntoLisp<'e>>(&'e self, length: usize, init: T) -> Result<Vector> {
        let value = self.call(&subr::MAKE_VECTOR, (length, init))?;
        Ok(Vector::from_value_unchecked(value, length))
    }

    pub fn vector<'e, A: IntoLispArgs<'e>>(&'e self, args: A) -> Result<Value> {
        self.call(&subr::VECTOR, args)
    }
}
