use crate::parser::MacroInput;
use crate::utils::{FieldValidator, expand_content, get_crate_path};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{Result, parse2};

pub fn assistant_impl(input: TokenStream2) -> Result<TokenStream2> {
    let macro_input: MacroInput = parse2(input)?;
    let root = get_crate_path();

    let (content, name, tool_calls, refusal) = match macro_input {
        MacroInput::Simple(expr) => (Some(expr), None, None, None),
        MacroInput::KeyValue(kvs) => {
            let mut validator = FieldValidator::new(kvs);
            validator.validate_field(&["content", "name", "tool_calls", "refusal"])?;
            let content = validator.optional("content")?;
            let name = validator.optional("name")?;
            let tool_calls = validator.optional("tool_calls")?;
            let refusal = validator.optional("refusal")?;
            (content, name, tool_calls, refusal)
        }
    };

    let content = content.map_or_else(
        || quote! { std::option::Option::None },
        |c| {
            let expanded_content = expand_content(&root, c.to_token_stream());
            quote! { std::option::Option::Some(#expanded_content) }
        },
    );
    let name = name.map_or_else(
        || quote! { std::option::Option::None },
        |n| quote! { std::option::Option::Some(#n.to_string()) },
    );
    let tool_calls = tool_calls.map_or_else(
        || quote! { std::option::Option::None },
        |t| quote! { std::option::Option::Some(#t) },
    );
    let refusal = refusal.map_or_else(
        || quote! { std::option::Option::None },
        |r| quote! { std::option::Option::Some(#r.to_string()) },
    );

    Ok(quote! {
        #root::modules::chat::types::ChatCompletionMessageParam::Assistant(
            #root::modules::chat::types::ChatCompletionAssistantMessageParam {
                content: #content,
                name: #name,
                tool_calls: #tool_calls,
                refusal: #refusal,
            },
        )
    })
}
