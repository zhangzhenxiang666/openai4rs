use crate::parser::MacroInput;
use crate::utils::{FieldValidator, expand_content, get_crate_path};
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::spanned::Spanned;
use syn::{Result, parse2};

pub fn tool_impl(input: TokenStream2) -> Result<TokenStream2> {
    let span = input.span();
    let macro_input: MacroInput = parse2(input)?;
    let root = get_crate_path();

    let (tool_call_id, content) = match macro_input {
        MacroInput::Simple(_) => {
            let msg = "The `tool!` macro requires key-value pairs, e.g., `tool!(tool_call_id: \"...\", content: \"...\")`.";
            return Err(syn::Error::new(span, msg));
        }
        MacroInput::KeyValue(kvs) => {
            let mut validator = FieldValidator::new(kvs);
            validator.validate_field(&["tool_call_id", "content"])?;
            let tool_call_id = validator.required("tool_call_id", span)?;
            let content = validator.required("content", span)?;
            (tool_call_id, content)
        }
    };

    let content = expand_content(&root, quote! {#content});

    Ok(quote! {
        #root::modules::chat::types::ChatCompletionMessageParam::Tool(
            #root::modules::chat::types::ChatCompletionToolMessageParam {
                tool_call_id: #tool_call_id.to_string(),
                content: #content,
            },
        )
    })
}
