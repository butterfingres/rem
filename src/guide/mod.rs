macro_rules! mod_doc {
    ($name:ident, $path:literal) => {
        pub mod $name {
            #![doc = include_str!($path)]
        }
    };
}
mod_doc!(calling_lisp, "../../doc/calling-lisp.md");
mod_doc!(custom_types, "../../doc/custom-types.md");
mod_doc!(errors, "../../doc/errors.md");
mod_doc!(functions, "../../doc/functions.md");
mod_doc!(hello, "../../doc/hello.md");
mod_doc!(module, "../../doc/module.md");
mod_doc!(open_channel, "../../doc/open-channel.md");
mod_doc!(overview, "../../doc/overview.md");
mod_doc!(reloading, "../../doc/reloading.md");
mod_doc!(testing, "../../doc/testing.md");
mod_doc!(type_conversions, "../../doc/type-conversions.md");
