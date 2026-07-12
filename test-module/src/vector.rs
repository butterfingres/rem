//! Testing a custom vector (user-ptr).

use rem::{defun, Env, IntoLisp, Result, Value};

struct Vector {
    pub x: i64,
    pub y: i64,
}

custom_types! {
    Vector;
}

#[defun(name = SwapComponents)]
fn swap_components<'e>(env: &'e Env, mut v: Value<'e>) -> Result<Value<'e>> {
    let vec: &mut Vector = unsafe { v.get_mut(env)? };
    vec.x ^= vec.y;
    vec.y ^= vec.x;
    vec.x ^= vec.y;
    Ok(v)
}

#[defun(user_ptr(direct), name = Make)]
fn make(x: i64, y: i64) -> Result<Vector> {
    Ok(Vector { x, y })
}

// Same with the above, but manually.
#[defun(name = Make1)]
fn make1(x: i64, y: i64) -> Result<Box<Vector>> {
    Ok(Box::new(Vector { x, y }))
}

#[defun(name = ToList)]
fn to_list<'e>(env: &'e Env, v: Value<'_>) -> Result<Value<'e>> {
    v.into_rust::<&Vector>(env)?;
    let v: &Vector = v.into_rust(env)?;
    let x = v.x.into_lisp(env)?;
    let y = v.y.into_lisp(env)?;
    env.list(&[x, y])
}

#[defun(user_ptr(direct), name = Add)]
fn add<'e>(env: &'e Env, a: Value<'e>, b: Value<'e>) -> Result<Vector> {
    let a: &Vector = a.into_rust(env)?;
    let b: &Vector = b.into_rust(env)?;
    let (x, y) = (b.x + a.x, b.y + a.y);
    Ok(Vector { x, y })
}

#[defun(name = ScaleMutably)]
fn scale_mutably<'e>(env: &'e Env, times: i64, mut v: Value<'e>) -> Result<()> {
    let v = unsafe { v.get_mut::<Vector>(env)? };
    v.x *= times;
    v.y *= times;
    Ok(())
}

pub fn init(env: &Env) -> Result<()> {
    env.lambda(&SwapComponents, None)?.fset(env, "t/vector-swap-components")?;
    env.lambda(&Make, None)?.fset(env, "t/vector-make")?;
    env.lambda(&Make1, None)?.fset(env, "t/vector-make1")?;
    env.lambda(&ToList, None)?.fset(env, "t/vector-to-list")?;
    env.lambda(&Add, None)?.fset(env, "t/vector-add")?;
    env.lambda(&ScaleMutably, None)?.fset(env, "t/vector-scale-mutably")?;
    Ok(())
}
