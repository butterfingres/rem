use rem::{Env, Result};

rem::plugin_is_GPL_compatible!();

mod test_channel;

#[rem::module]
fn t28(env: &Env) -> Result<()> {
    test_channel::init(env)?;

    Ok(())
}
