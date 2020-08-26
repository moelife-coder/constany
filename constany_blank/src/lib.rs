//! ## Please refer to `constany_stage_one` document.
//! This crate is a blank implementation for `constany` to satisfy grammar checker and avoid conflict.

extern crate proc_macro;

use crate::proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn const_fn(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn main_fn(_: TokenStream, item: TokenStream) -> TokenStream {
    item
}
