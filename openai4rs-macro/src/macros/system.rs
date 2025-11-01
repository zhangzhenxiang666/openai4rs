use crate::parser::MacroInput;
use crate::utils::{FieldValidator, expand_content, get_crate_path};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::spanned::Spanned;
use syn::{Result, parse2};

pub fn system_impl(input: TokenStream2) -> Result<TokenStream2> {
    let span = input.span();
    let macro_input: MacroInput = parse2(input)?;
    let root = get_crate_path();

    let (content, name) = match macro_input {
        MacroInput::Simple(expr) => (expr, None),
        MacroInput::KeyValue(kvs) => {
            let mut validator = FieldValidator::new(kvs);
            validator.validate_field(&["content", "name"])?;
            let content = validator.required("content", span)?;
            let name = validator.optional("name")?;
            (content, name)
        }
    };

    let content = expand_content(&root, content.to_token_stream());
    let name = name.map_or_else(
        || quote!(std::option::Option::None),
        |n| quote!(std::option::Option::Some(#n.to_string())),
    );

    Ok(quote! {
        #root::modules::chat::types::ChatCompletionMessageParam::System(
            #root::modules::chat::types::ChatCompletionSystemMessageParam {
                content: #content,
                name: #name,
            }
        )
    })
}
