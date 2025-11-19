use super::params::EmbeddingsParam;
use super::types::EmbeddingResponse;
use crate::OpenAIError;
use crate::common::types::{InParam, RetryCount, Timeout};
use crate::service::{
    HttpClient,
    request::{RequestBuilder, RequestSpec},
};

/// 处理嵌入请求，用于生成文本的向量表示。
pub struct Embeddings {
    http_client: HttpClient,
}

impl Embeddings {
    pub(crate) fn new(http_client: HttpClient) -> Embeddings {
        Embeddings { http_client }
    }

    /// 为提供的输入文本创建嵌入。
    ///
    /// 此方法向API发送请求并返回输入文本的向量表示。
    ///
    /// # 参数
    ///
    /// * `param` - 嵌入请求的一组参数，例如模型和输入文本。
    ///   可以使用 `embeddings_request` 创建。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use openai4rs::*;
    /// use openai4rs::embeddings::EmbeddingsParam;
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let request = EmbeddingsParam::new("text-embedding-ada-002", "Hello, world!");
    ///     let response = client.embeddings().create(request).await?;
    ///     println!("{:#?}", response);
    ///     Ok(())
    /// }
    /// ```
    pub async fn create(&self, param: EmbeddingsParam) -> Result<EmbeddingResponse, OpenAIError> {
        let inner = param.take();

        let http_params = RequestSpec::new(
            |config| format!("{}/embeddings", config.base_url()),
            |config, builder| {
                Self::apply_request_settings(builder, inner);
                builder.bearer_auth(config.api_key());
            },
        );
        self.http_client.post_json(http_params).await
    }
}

impl Embeddings {
    fn apply_request_settings(builder: &mut RequestBuilder, params: InParam) {
        let body = params
            .body
            .unwrap_or_else(|| panic!("Unknown internal error, please submit an issue."));

        builder.body_fields(body);

        *builder.headers_mut() = params.headers;

        if let Some(time) = params.extensions.get::<Timeout>() {
            builder.timeout(time.0);
        }

        if let Some(retry) = params.extensions.get::<RetryCount>() {
            builder.extensions_mut().insert(retry.clone());
        }
    }
}
