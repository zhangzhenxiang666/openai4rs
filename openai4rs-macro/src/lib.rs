mod macros;
mod parser;
mod utils;

use proc_macro::TokenStream;
use syn::parse_macro_input;

/// Creates a `ChatCompletionMessageParam::System` message.
///
/// This macro supports two forms:
/// 1. Simple form: `system!("content")`
/// 2. Key-value form: `system!(content: "content", name: "name")`
///
/// The `name` field is optional in the key-value form.
#[proc_macro]
pub fn system(input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as proc_macro2::TokenStream);
    match macros::system::system_impl(st) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Creates a `ChatCompletionMessageParam::User` message.
///
/// This macro supports two forms:
/// 1. Simple form: `user!("content")`
/// 2. Key-value form: `user!(content: "content", name: "name")`
///
/// The `name` field is optional in the key-value form.
///
#[proc_macro]
pub fn user(input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as proc_macro2::TokenStream);
    match macros::user::user_impl(st) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Creates a `ChatCompletionMessageParam::Assistant` message.
///
/// This macro supports two forms:
/// 1. Simple form: `assistant!("content")`
/// 2. Key-value form: `assistant!(content: "content", name: "name", tool_calls: vec![...])`
///
/// All fields are optional in the key-value form.
#[proc_macro]
pub fn assistant(input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as proc_macro2::TokenStream);
    match macros::assistant::assistant_impl(st) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Creates a `ChatCompletionMessageParam::Tool` message.
///
/// This macro requires key-value form with both `tool_call_id` and `content` fields.
#[proc_macro]
pub fn tool(input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as proc_macro2::TokenStream);
    match macros::tool::tool_impl(st) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}

/// Creates a `chat::Content` value.
///
/// This macro supports both simple string content and complex JSON content.
#[proc_macro]
pub fn content(input: TokenStream) -> TokenStream {
    let st = parse_macro_input!(input as proc_macro2::TokenStream);
    match macros::content::content_impl(st) {
        Ok(ts) => ts.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
