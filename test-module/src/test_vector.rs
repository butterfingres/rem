//! Testing bindings for vector functions (vec_get, vec_set, vec_size).

use rem::{defun, Env, Result, Value, Vector};

#[defun(mod_in_name = false)]
fn vec_size(v: Vector) -> Result<usize> {
    Ok(v.len())
}

#[defun(mod_in_name = false)]
fn vec_get<'e>(env: &'e Env, v: Vector<'e>, i: i64) -> Result<Value<'e>> {
    v.get(env, i as usize)
}

#[defun(mod_in_name = false)]
fn vec_set(env: &Env, v: Vector, i: i64, value: Value) -> Result<()> {
    v.set(env, i as usize, value)
}

#[defun(mod_in_name = false)]
fn identity_if_vector(v: Vector) -> Result<Vector> {
    Ok(v)
}

#[defun(mod_in_name = false)]
fn stringify_num_vector<'e>(env: &'e Env, v: Vector<'e>) -> Result<Vector<'e>> {
    for i in 0..v.len() {
        let x: i64 = v.get(env, i)?;
        v.set(env, i, format!("{}", x))?;
    }
    Ok(v)
}

#[defun(mod_in_name = false)]
fn make_vector<'e>(env: &'e Env, length: usize, init: Value<'e>) -> Result<Vector<'e>> {
    env.make_vector(length, init)
}
