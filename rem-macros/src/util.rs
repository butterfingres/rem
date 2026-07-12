use std::fmt::Display;

use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::{ToTokens, TokenStreamExt};
use syn::{ext::IdentExt, Ident};

// TODO: Add more extensively checks and transformations to make this more "idiomatic".
pub fn lisp_name(id: &Ident) -> String {
    id.unraw().to_string().replace("_", "-")
}

// pub fn concat(lhs: &str, rhs: &Ident) -> Ident {
//     Ident::new(&format!("{}{}", lhs, rhs.unraw()), Span::call_site())
// }

pub fn arg(name: &str, i: usize) -> Ident {
    Ident::new(&format!("{}{}", name, i), Span::call_site())
}

pub fn report<T: ToTokens, U: Display>(errors: &mut TokenStream2, ts: T, msg: U) {
    errors.append_all(syn::Error::new_spanned(ts, msg).to_compile_error());
}
