use super::*;

impl<'e> FromLisp<'e> for f64 {
    fn from_lisp(value: Value<'e>, env: &'e Env) -> Result<Self> {
        unsafe_raw_call!(env, extract_float, value.raw)
    }
}

impl IntoLisp<'_> for f64 {
    fn into_lisp(self, env: &Env) -> Result<Value<'_>> {
        unsafe_raw_call_value_unprotected!(env, make_float, self)
    }
}
