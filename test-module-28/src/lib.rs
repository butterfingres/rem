use rem::{Env, Result};

rem::plugin_is_GPL_compatible!();

mod test_channel;

#[rem::module(name(fn), separator = "/", mod_in_name = false)]
fn t28(_env: &Env) -> Result<()> {
    Ok(())
}
