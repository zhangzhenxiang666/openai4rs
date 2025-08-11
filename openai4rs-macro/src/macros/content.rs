use crate::utils::{expand_content, get_crate_path};
use proc_macro2::TokenStream as TokenStream2;
use syn::Result;

pub fn content_impl(input: TokenStream2) -> Result<TokenStream2> {
    let root = get_crate_path();
    Ok(expand_content(&root, input))
}
