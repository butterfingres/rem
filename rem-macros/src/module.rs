use darling::{self, ast::NestedMeta, FromMeta};
use proc_macro2::{TokenStream as TokenStream2};
use quote::quote;
use syn::{ItemFn};

use crate::util;

#[derive(Debug, Default)]
enum Name {
    /// Use crate's name.
    #[default]
    Crate,
    /// Use initializer function's name.
    Fn,
    /// Explicitly specify a name.
    Str(String),
}

#[derive(Debug, FromMeta)]
struct ModuleOpts {
    /// Name of this module (feature name).
    #[darling(default)]
    name: Name,
}

#[derive(Debug)]
pub struct Module {
    def: ItemFn,
    opts: ModuleOpts,
}

/// We don't use the derived impl provided by darling, since we want a different syntax.
/// See https://github.com/TedDriggs/darling/issues/74.
impl FromMeta for Name {
    fn from_list(outer: &[NestedMeta]) -> darling::Result<Name> {
        match outer.len() {
            0 => Err(darling::Error::too_few_items(1)),
            1 => {
                let elem = &outer[0];
                match elem {
                    NestedMeta::Meta(syn::Meta::Path(path)) => {
                        match path.segments.last().unwrap().ident.to_string().as_ref() {
                            "fn" => Ok(Name::Fn),
                            "crate" => Ok(Name::Crate),
                            _ => Err(darling::Error::custom("Expected crate/fn").with_span(path)),
                        }
                    }
                    NestedMeta::Lit(syn::Lit::Str(lit)) => Ok(Name::Str(lit.value())),
                    _ => {
                        Err(darling::Error::custom("Expected crate/fn or a string").with_span(elem))
                    }
                }
            }
            _ => Err(darling::Error::too_many_items(1)),
        }
    }

    fn from_string(lit: &str) -> darling::Result<Name> {
        Ok(Name::Str(lit.to_owned()))
    }
}

impl Module {
    pub fn parse(attr_args: Vec<NestedMeta>, fn_item: ItemFn) -> Result<Self, TokenStream2> {
        let opts: ModuleOpts = match ModuleOpts::from_list(&attr_args) {
            Ok(v) => v,
            Err(e) => return Err(e.write_errors()),
        };
        Ok(Self { opts, def: fn_item })
    }

    pub fn render(&self) -> TokenStream2 {
        let define_init = self.gen_init();
        let register_init = Self::gen_registrator();
        let define_hook = &self.def;
        quote! {
            #define_hook
            #define_init
            #register_init
        }
    }

    pub fn gen_registrator() -> TokenStream2 {
        let init = Self::init_ident();
        quote! {
            ::rem::__module_init!(#init);
        }
    }

    pub fn gen_init(&self) -> TokenStream2 {
        let init = Self::init_ident();
        let env = quote!(env);
        let hook = &self.def.sig.ident;
        let feature = match &self.opts.name {
            Name::Crate => quote!({
                const PATH: &str = module_path!();
                const LEN: usize = {
                    let mut i = 0;
                    while i < PATH.len() {
                        if PATH.as_bytes()[i] == b':' {
                            break;
                        }
                        i += 1;
                    }
                    i
                };
                const BUF: [u8; LEN] = {
                    let mut buf = [0; LEN];
                    let mut i = 0;
                    while i < LEN {
                        buf[i] = match PATH.as_bytes()[i] {
                            b'_' => b'-',
                            ch => ch,
                        };
                        i += 1;
                    }
                    buf
                };
                const { unsafe { std::str::from_utf8_unchecked(&BUF) } }
            }),
            Name::Str(name) => quote!(#name),
            Name::Fn => {
                let name = util::lisp_name(hook);
                quote!(#name)
            }
        };
        quote! {
            #[allow(non_snake_case)]
            fn #init(#env: &::rem::Env) -> ::rem::Result<::rem::Value<'_>> {
                const FEATURE: &str = #feature;
                #hook(#env)?;
                #env.provide(&FEATURE)
            }
        }
    }

    fn init_ident() -> TokenStream2 {
        quote!(__emrs_auto_init__)
    }
}
