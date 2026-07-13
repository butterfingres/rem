//! Testing bindings for vector functions (vec_get, vec_set, vec_size).

use rem::{defun, Env, Result, Value, Vector};

#[defun(name = VecSize)]
fn vec_size(v: Vector) -> Result<usize> {
    Ok(v.len())
}

#[defun(name = VecGet)]
fn vec_get<'e>(env: &'e Env, v: Vector<'e>, i: i64) -> Result<Value<'e>> {
    v.get(env, i as usize)
}

#[defun(name = VecSet)]
fn vec_set(env: &Env, v: Vector, i: i64, value: Value) -> Result<()> {
    v.set(env, i as usize, value)
}

#[defun(name = IdentityIfVector)]
fn identity_if_vector(v: Vector) -> Result<Vector> {
    Ok(v)
}

#[defun(name = StringifyNumVector)]
fn stringify_num_vector<'e>(env: &'e Env, v: Vector<'e>) -> Result<Vector<'e>> {
    for i in 0..v.len() {
        let x: i64 = v.get(env, i)?;
        v.set(env, i, format!("{}", x))?;
    }
    Ok(v)
}

#[defun(name = MakeVector)]
fn make_vector<'e>(env: &'e Env, length: usize, init: Value<'e>) -> Result<Vector<'e>> {
    env.make_vector(length, init)
}

pub fn init(env: &Env) -> Result<()> {
    env.lambda(&VecSize, None)?.fset("t/vec-size")?;
    env.lambda(&VecGet, None)?.fset("t/vec-get")?;
    env.lambda(&VecSet, None)?.fset("t/vec-set")?;
    env.lambda(&IdentityIfVector, None)?.fset("t/identity-if-vector")?;
    env.lambda(&StringifyNumVector, None)?.fset("t/stringify-num-vector")?;
    env.lambda(&MakeVector, None)?.fset("t/make-vector")?;

    Ok(())
}
