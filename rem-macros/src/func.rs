use std::ops::Range;

use darling::{FromMeta, ast::NestedMeta};
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{TokenStreamExt, quote, quote_spanned};
use syn::{
    self, FnArg, Ident, ItemFn, PatType, ReturnType, Signature, Type, TypePath, TypeReference,
    spanned::Spanned,
};

use crate::util::{self, report};

#[derive(Debug)]
enum Arg {
    Env { span: Span },
    Val { span: Span, access: Access, nth: usize },
}

/// Kinds of argument.
#[derive(Copy, Clone, Debug)]
enum Access {
    Owned,
    Ref,
    RefMut,
}

/// Kinds of `user-ptr` embedding.
#[derive(Debug)]
enum UserPtr {
    /// Embedding through a [`RefCell`]. This is suitable for common use cases, where module
    /// functions can borrow the underlying data back for read/write. It is safe because Lisp
    /// threads are subjected to the GIL. [`BorrowError`]/[`BorrowMutError`] may be signaled at
    /// runtime, depending on how module functions call back into the Lisp runtime.
    ///
    /// [`RefCell`]: https://doc.rust-lang.org/std/cell/struct.RefCell.html
    /// [`BorrowError`]: https://doc.rust-lang.org/std/cell/struct.BorrowError.html
    /// [`BorrowMutError`]: https://doc.rust-lang.org/std/cell/struct.BorrowMutError.html
    RefCell,
    /// Embedding through a [`RwLock`]. Suitable for sharing data between module functions (on Lisp
    /// threads, with [`Env`] access) and pure Rust code (on background threads, without [`Env`]
    /// access).
    ///
    /// [`RwLock`]: https://doc.rust-lang.org/std/sync/struct.RwLock.html
    RwLock,
    /// Embedding through a [`Mutex`]. Suitable for sharing data between module functions (on Lisp
    /// threads, with [`Env`] access) and pure Rust code (on background threads, without [`Env`]
    /// access).
    ///
    /// [`Mutex`]: https://doc.rust-lang.org/std/sync/struct.Mutex.html
    Mutex,
    /// Embedding a `Transfer` value directly. Suitable for immutable data that will only be read
    /// back (not written to) by module functions (writing requires `unsafe` access, and is
    /// discouraged).
    Direct,
}

#[derive(Debug, FromMeta)]
struct FuncOpts {
    /// How the return value should be embedded in Lisp as a `user-ptr`. `None` means no embedding.
    #[darling(default)]
    user_ptr: Option<UserPtr>,
    #[darling(default)]
    name: Option<Ident>,
}

#[derive(Debug)]
pub struct LispFunc {
    /// The original Rust definition.
    def: ItemFn,
    /// Relevant info about the arguments in Rust.
    args: Vec<Arg>,
    /// Function's arities in Lisp.
    arities: Range<usize>,
    /// Span of the return type. This helps with error reporting.
    output_span: Span,
    opts: FuncOpts,
}

/// We don't use the derived impl provided by darling, since we want a different syntax.
/// See https://github.com/TedDriggs/darling/issues/74.
impl FromMeta for UserPtr {
    fn from_word() -> darling::Result<UserPtr> {
        Ok(UserPtr::RefCell)
    }

    fn from_list(outer: &[NestedMeta]) -> darling::Result<UserPtr> {
        match outer {
            [NestedMeta::Meta(syn::Meta::Path(path))] => {
                match path.segments.last().unwrap().ident.to_string().as_ref() {
                    "refcell" => Ok(UserPtr::RefCell),
                    "mutex" => Ok(UserPtr::Mutex),
                    "rwlock" => Ok(UserPtr::RwLock),
                    "direct" => Ok(UserPtr::Direct),
                    _ => Err(darling::Error::custom("Unknown kind of embedding").with_span(path)),
                }
            }
            [elem] => Err(darling::Error::custom("Expected an identifier").with_span(elem)),
            _ => Err(darling::Error::too_few_items(1)),
        }
    }
}

impl LispFunc {
    pub fn parse(attr_args: Vec<NestedMeta>, fn_item: ItemFn) -> Result<Self, TokenStream2> {
        let opts: FuncOpts = match FuncOpts::from_list(&attr_args) {
            Ok(v) => v,
            Err(e) => return Err(e.write_errors()),
        };
        let (args, arities, output_span) = check_signature(&fn_item.sig)?;
        let def = fn_item;
        Ok(Self { def, args, arities, output_span, opts })
    }

    pub fn render(&self) -> TokenStream2 {
        let define_exporter = self.gen_exporter();
        let define_func = &self.def;
        quote! {
            #define_func
            #define_exporter
        }
    }

    /// Generates the wrapper function, which decodes input arguments and encodes return value.
    pub fn gen_wrapper(&self) -> TokenStream2 {
        let mut args = TokenStream2::new();
        // Inlined references do not live long enough. We need bindings for them.
        let mut bindings = TokenStream2::new();
        let env = Ident::new("env", Span::call_site());
        let vals = Ident::new("vals", Span::call_site());
        for arg in &self.args {
            match *arg {
                Arg::Env { span } => {
                    // TODO: Find a way not to define inner function, somehow, otherwise the reported
                    // error is confusing (i.e expecting Env, found &Env).
                    args.append_all(quote_spanned!(span=> #env,))
                }
                Arg::Val { span, access, nth, .. } => {
                    let name = util::arg("arg", nth);
                    // TODO: Create a slice of `emacs_value` once and iterate through it, instead of
                    // using `get_arg`, which creates a slice each call.
                    bindings.append_all(match access {
                        Access::Owned => quote_spanned! {
                            span => let #name = #vals[#nth].into_rust(#env)?;
                        },
                        // TODO: Support RwLock/Mutex (for the use case of sharing data with
                        // background Rust threads).
                        // TODO: Support direct access.
                        Access::Ref => quote_spanned! {
                            span => let #name = &*#vals[#nth].into_ref(#env)?;
                        },
                        Access::RefMut => quote_spanned! {span=>
                            let #name = &mut *#vals[#nth].into_ref_mut(#env)?;
                        },
                    });
                    args.append_all(quote_spanned!(span=> #name,));
                }
            }
        }
        let maybe_embed = match &self.opts.user_ptr {
            None => TokenStream2::new(),
            Some(user_ptr) => match user_ptr {
                UserPtr::RefCell => quote_spanned! {self.output_span=>
                    let output = ::std::boxed::Box::new(::std::cell::RefCell::new(output));
                },
                UserPtr::RwLock => quote_spanned! {self.output_span=>
                    let output = ::std::boxed::Box::new(::std::sync::RwLock::new(output));
                },
                UserPtr::Mutex => quote_spanned! {self.output_span=>
                    let output = ::std::boxed::Box::new(::std::sync::Mutex::new(output));
                },
                UserPtr::Direct => quote_spanned! {self.output_span=>
                    let output = ::std::boxed::Box::new(output);
                },
            },
        };
        // XXX: output can be (), but we can't easily know when.
        let into_lisp = quote_spanned! {self.output_span=>
            #[allow(clippy::unit_arg)]
            ::rem::IntoLisp::into_lisp(output, #env)
        };
        let inner = &self.def.sig.ident;
        let wrapper_struct = self.opts.name.clone().unwrap_or_else(|| {
            let ident = inner.to_string();

            let mut buf = String::with_capacity(ident.len());
            let mut next_upper = true;
            for ch in ident.chars() {
                match ch {
                    '_' => {
                        next_upper = true;
                    }
                    _ => {
                        if next_upper {
                            for ch in ch.to_uppercase() {
                                buf.push(ch);
                            }
                            next_upper = false;
                        } else {
                            buf.push(ch);
                        }
                    }
                }
            }
            Ident::new(&buf, inner.span())
        });

        let min = self.arities.start;
        let max = self.arities.end;

        quote! {
            pub struct #wrapper_struct;
            impl<'e> ::rem::func::LispFn<'e> for #wrapper_struct {
                const MIN_ARITY: ::std::primitive::usize = #min;
                const MAX_ARITY: ::std::option::Option<::std::primitive::usize> = ::std::option::Option::Some(#max);

                fn call(&self, #env: &'e ::rem::Env, #vals: &[::rem::Value<'e>]) -> rem::Result<rem::Value<'e>> {
                    #bindings
                    let output = #inner(#args)?;
                    #maybe_embed
                    #into_lisp
                }
            }
        }
    }

    /// Generates the exporter function. It will be called in `emacs_module_init` to bind the Lisp
    /// symbol to the defined extern function.
    pub fn gen_exporter(&self) -> TokenStream2 {
        let define_wrapper = self.gen_wrapper();
        quote! {
            #define_wrapper
        }
    }
}

fn check_signature(sig: &Signature) -> Result<(Vec<Arg>, Range<usize>, Span), TokenStream2> {
    let mut i: usize = 0;
    let mut err = TokenStream2::new();
    let mut has_env = false;
    let mut args: Vec<Arg> = vec![];
    let errors = &mut err;
    for fn_arg in &sig.inputs {
        match fn_arg {
            FnArg::Typed(PatType { ty, .. }) => {
                let span = fn_arg.span();
                args.push(if is_env(ty) {
                    match ty.as_ref() {
                        Type::Reference(_) => (),
                        _ => report(errors, fn_arg, "Can only take an &Env, not an Env"),
                    }
                    if has_env {
                        report(errors, fn_arg, "&Env must be passed only once")
                    }
                    has_env = true;
                    Arg::Env { span }
                } else {
                    let access = match ty.as_ref() {
                        Type::Reference(TypeReference { mutability, .. }) => match mutability {
                            Some(_) => Access::RefMut,
                            None => Access::Ref,
                        },
                        _ => Access::Owned,
                    };
                    let a = Arg::Val { span, access, nth: i };
                    i += 1;
                    a
                });
            }
            FnArg::Receiver(_) => report(errors, fn_arg, "Cannot take self argument"),
        }
    }
    // TODO: Make the Span span the whole return type.
    let output_span = match &sig.output {
        ReturnType::Type(_, ty) => ty.span(),
        _ => {
            report(errors, sig.fn_token, "Must return rem::Result<T> where T: IntoLisp<'_>");
            sig.fn_token.span()
        }
    };
    if err.is_empty() { Ok((args, Range { start: i, end: i }, output_span)) } else { Err(err) }
}

// XXX
fn is_env(ty: &Type) -> bool {
    match ty {
        Type::Reference(TypeReference { elem, .. }) => is_env(elem),
        Type::Path(TypePath { qself: None, path }) => {
            let str_path = format!("{}", quote!(#path));
            str_path.ends_with("Env")
        }
        _ => false,
    }
}
