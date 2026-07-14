//! Testing a custom vector (user-ptr).

use rem::{defun, Env, IntoLisp, Result, Value};

struct Vector2d {
    pub x: i64,
    pub y: i64,
}

custom_types! {
    Vector2d;
}

#[defun]
fn swap_components<'e>(env: &'e Env, mut v: Value<'e>) -> Result<Value<'e>> {
    let vec: &mut Vector2d = unsafe { v.get_mut(env)? };
    vec.x ^= vec.y;
    vec.y ^= vec.x;
    vec.x ^= vec.y;
    Ok(v)
}

#[defun(user_ptr(direct))]
fn make(x: i64, y: i64) -> Result<Vector2d> {
    Ok(Vector2d { x, y })
}

// Same with the above, but manually.
#[defun]
fn make1(x: i64, y: i64) -> Result<Box<Vector2d>> {
    Ok(Box::new(Vector2d { x, y }))
}

#[defun]
fn to_list<'e>(env: &'e Env, v: Value<'_>) -> Result<Value<'e>> {
    v.into_rust::<&Vector2d>(env)?;
    let v: &Vector2d = v.into_rust(env)?;
    let x = v.x.into_lisp(env)?;
    let y = v.y.into_lisp(env)?;
    env.list(&[x, y])
}

#[defun(user_ptr(direct))]
fn add<'e>(env: &'e Env, a: Value<'e>, b: Value<'e>) -> Result<Vector2d> {
    let a: &Vector2d = a.into_rust(env)?;
    let b: &Vector2d = b.into_rust(env)?;
    let (x, y) = (b.x + a.x, b.y + a.y);
    Ok(Vector2d { x, y })
}

#[defun]
fn scale_mutably<'e>(env: &'e Env, times: i64, mut v: Value<'e>) -> Result<()> {
    let v = unsafe { v.get_mut::<Vector2d>(env)? };
    v.x *= times;
    v.y *= times;
    Ok(())
}

pub fn init(env: &Env) -> Result<()> {
    env.lambda(&SwapComponents, None)?.fset("t/vector-2d-swap-components")?;
    env.lambda(&Make, None)?.fset("t/vector-2d-make")?;
    env.lambda(&Make1, None)?.fset("t/vector-2d-make1")?;
    env.lambda(&ToList, None)?.fset("t/vector-2d-to-list")?;
    env.lambda(&Add, None)?.fset("t/vector-2d-add")?;
    env.lambda(&ScaleMutably, None)?.fset("t/vector-2d-scale-mutably")?;
    Ok(())
}
