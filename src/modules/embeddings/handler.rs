use std::time::Duration;

use super::params::{IntoRequestParams, RequestParams};
use super::types::EmbeddingResponse;
use crate::{HttpClient, OpenAIError, RequestBuilder, service::request::RequestSpec};

/// Handles embedding requests for generating vector representations of text.
pub struct Embeddings {
    http_client: HttpClient,
}

impl Embeddings {
    pub fn new(http_client: HttpClient) -> Embeddings {
        Embeddings { http_client }
    }

    /// Creates embeddings for the provided input text.
    ///
    /// This method sends a request to the API and returns vector representations
    /// of the input text.
    ///
    /// # Arguments
    ///
    /// * `params` - A set of parameters for the embedding request, such as the model and input text.
    ///   Can be created using `embeddings_request`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use openai4rs::*;
    /// use openai4rs::embeddings::params::embeddings_request;
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let request = embeddings_request("text-embedding-ada-002", "Hello, world!");
    ///     let response = client.embeddings().create(request).await?;
    ///     println!("{:#?}", response);
    ///     Ok(())
    /// }
    /// ```
    pub async fn create<T>(&self, params: T) -> Result<EmbeddingResponse, OpenAIError>
    where
        T: IntoRequestParams,
    {
        let params = params.into_request_params();
        let retry_count = params.retry_count.unwrap_or(0);
        let http_params = RequestSpec::new(
            |config| format!("{}/embeddings", config.base_url()),
            |config, builder| {
                Self::apply_request_settings(builder, params);
                builder.bearer_auth(config.api_key());
            },
            retry_count,
            None,
        );
        self.http_client.post_json(http_params).await
    }
}

impl Embeddings {
    fn apply_request_settings(builder: &mut RequestBuilder, params: RequestParams) {
        if let Ok(serde_json::Value::Object(obj)) = serde_json::to_value(&params) {
            builder.body_fields(obj.into_iter().collect());
        }

        if let Some(headers) = params.extra_headers {
            headers.into_iter().for_each(|(k, v)| {
                builder.header(k, v);
            });
        }

        if let Some(query) = params.extra_query {
            query.into_iter().for_each(|(k, v)| {
                builder.query(k, v);
            });
        }

        if let Some(extra_body) = params.extra_body {
            extra_body.into_iter().for_each(|(k, v)| {
                builder.body_field(k, v);
            });
        }

        if let Some(timeout) = params.timeout_seconds {
            builder.timeout(Duration::from_secs(timeout));
        }

        if let Some(user_agent) = params.user_agent {
            builder.header("user-agent", user_agent);
        }
    }
}
