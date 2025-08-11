use proc_macro2::Ident;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, Result, Token};

// Represents a key-value pair like `content: "hello"` or `name = "user"`
#[derive(Clone)]
pub(crate) struct KeyValue {
    pub key: Ident,
    pub value: Expr,
}

impl Parse for KeyValue {
    fn parse(input: ParseStream) -> Result<Self> {
        let key = input.parse::<Ident>()?;

        if input.peek(Token![:]) {
            input.parse::<Token![:]>()?;
        } else {
            input.parse::<Token![=]>()?;
        }

        let value = input.parse::<Expr>()?;

        Ok(KeyValue { key, value })
    }
}

// Represents the possible inputs to our message macros
#[derive(Clone)]
pub(crate) enum MacroInput {
    // A single string literal, e.g., `user!("hello")`
    Simple(Expr),
    // A list of key-value pairs, e.g., `user!(content: "hello", name: "user")`
    KeyValue(Vec<KeyValue>),
}

impl Parse for MacroInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "Expected at least one argument",
            ));
        }

        if input.peek(syn::Ident) && (input.peek2(Token![:]) || input.peek2(Token![=])) {
            let mut kvs = Vec::new();
            while !input.is_empty() {
                kvs.push(input.parse()?);
                if !input.is_empty() {
                    input.parse::<Token![,]>()?;
                }
            }
            return Ok(MacroInput::KeyValue(kvs));
        }

        let expr: Expr = input.parse().map_err(|_| {
            syn::Error::new(input.span(), "Input cannot be empty or invalid expression")
        })?;

        if !input.is_empty() {
            return Err(syn::Error::new(
                input.span(),
                "Unexpected token. A simple message must be a single expression. For multiple fields, use key-value pairs.",
            ));
        }

        Ok(MacroInput::Simple(expr))
    }
}
