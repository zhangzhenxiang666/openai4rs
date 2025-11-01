use crate::parser::KeyValue;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::quote;
use syn::{Expr, Result};

/// Helper to get the correct path to the `openai4rs` crate.
///
/// This function dynamically determines the correct path to use when referencing
/// the `openai4rs` crate from within a procedural macro. It handles both cases
/// where the macro is used within the `openai4rs` crate itself and when it's
/// used as a dependency in another crate.
pub fn get_crate_path() -> proc_macro2::TokenStream {
    let found_crate = crate_name("openai4rs").expect("`openai4rs` is not present in `Cargo.toml`");
    match found_crate {
        FoundCrate::Itself => quote!(crate),
        FoundCrate::Name(name) => {
            let ident = syn::Ident::new(&name, proc_macro2::Span::call_site());
            quote!(::#ident)
        }
    }
}

/// Expands the content expression into the appropriate `chat::Content` variant.
///
/// This function replicates the logic of the original `content!` macro, converting
/// a JSON expression into the correct `chat::Content` enum value based on its type.
///
/// # Arguments
///
/// * `root` - The resolved path to the `openai4rs` crate.
/// * `content_expr` - The content expression to expand.
///
/// # Returns
///
/// A `proc_macro2::TokenStream` containing the expanded content.
pub fn expand_content(
    root: &proc_macro2::TokenStream,
    content_expr: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote! {
        {
            let value = #root::serde_json::json!(#content_expr);
            match value {
                #root::serde_json::Value::Object(_) => #root::modules::chat::types::Content::Object(value),
                #root::serde_json::Value::String(s) => #root::modules::chat::types::Content::Text(s),
                #root::serde_json::Value::Array(_) => #root::modules::chat::types::Content::Object(value),
                #root::serde_json::Value::Number(n) => #root::modules::chat::types::Content::Text(n.to_string()),
                #root::serde_json::Value::Bool(b) => #root::modules::chat::types::Content::Text(b.to_string()),
                #root::serde_json::Value::Null => #root::modules::chat::types::Content::Text(String::from("null")),
            }
        }
    }
}

pub(crate) struct FieldValidator {
    kvs: Vec<KeyValue>,
}

impl FieldValidator {
    pub(crate) fn new(kvs: Vec<KeyValue>) -> Self {
        Self { kvs }
    }

    pub(crate) fn validate_field(&self, allowed_fields: &[&str]) -> Result<()> {
        for kv in &self.kvs {
            let k_string = kv.key.to_string();
            if !allowed_fields.contains(&k_string.as_str()) {
                return Err(syn::Error::new(
                    kv.key.span(),
                    format!(
                        "The field '{}' must be one of the following: {:?}",
                        k_string, allowed_fields
                    ),
                ));
            }
        }
        Ok(())
    }

    pub(crate) fn required(&mut self, field: &str, span: Span) -> Result<Expr> {
        match self.find_one(field)? {
            Some(v) => Ok(v),
            None => Err(syn::Error::new(
                span,
                format!("Missing required field: {:?}", field),
            )),
        }
    }

    pub(crate) fn optional(&mut self, field: &str) -> Result<Option<Expr>> {
        self.find_one(field)
    }

    fn find_one(&mut self, field: &str) -> Result<Option<Expr>> {
        let mut found: Option<Expr> = None;
        let mut indices = Vec::new();

        for (i, kv) in self.kvs.iter().enumerate() {
            if kv.key == field {
                indices.push(i);
            }
        }

        if indices.len() > 1 {
            let first_span = self.kvs[indices[0]].key.span();
            let second_span = self.kvs[indices[1]].key.span();
            let mut err = syn::Error::new(
                second_span,
                format!("Duplicate field `{}` specified.", field),
            );
            err.combine(syn::Error::new(first_span, "First definition is here."));
            return Err(err);
        }

        if let Some(index) = indices.into_iter().next() {
            let kv = self.kvs.remove(index);
            found = Some(kv.value);
        }

        Ok(found)
    }
}
