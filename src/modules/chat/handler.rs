use core::panic;

use super::params::ChatParam;
use super::types::{ChatCompletion, ChatCompletionChunk};
use crate::common::types::{InParam, RetryCount, Timeout};
use crate::error::OpenAIError;
use crate::service::client::HttpClient;
use crate::service::request::{RequestBuilder, RequestSpec};
use tokio_stream::wrappers::ReceiverStream;

/// 处理聊天完成请求，包括流式和非流式模式。
pub struct Chat {
    http_client: HttpClient,
}

impl Chat {
    pub(crate) fn new(http_client: HttpClient) -> Chat {
        Chat { http_client }
    }

    /// 创建一个聊天完成。
    ///
    /// 此方法向API发送请求，并在单个响应中返回完整的完成结果。
    ///
    /// # 参数
    ///
    /// * `param` - 聊天完成的一组参数，例如模型和消息。
    ///   可以使用 `ChatParam` 创建。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use openai4rs::*;
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let messages = vec![user!("What is Rust?")];
    ///     let request = ChatParam::new("Qwen/Qwen3-235B-A22B-Instruct-2507", &messages);
    ///     let response = client.chat().create(request).await?;
    ///     println!("{:#?}", response);
    ///     Ok(())
    /// }
    /// ```
    pub async fn create(&self, param: ChatParam) -> Result<ChatCompletion, OpenAIError> {
        let mut inner = param.take();
        inner
            .body
            .as_mut()
            .unwrap()
            .insert("stream".to_string(), serde_json::to_value(false).unwrap());

        let http_params = RequestSpec::new(
            |config| format!("{}/chat/completions", config.base_url()),
            move |config, request| {
                let mut builder = RequestBuilder::new(request);
                Self::apply_request_settings(&mut builder, inner);
                builder.bearer_auth(config.api_key());
                builder.take()
            },
        );

        self.http_client.post_json(http_params).await
    }

    /// 创建一个流式聊天完成。
    ///
    /// 此方法返回 `ChatCompletionChunk` 事件流。这对于实时显示生成的完成结果非常有用。
    ///
    /// # 参数
    ///
    /// * `param` - 聊天完成的一组参数，例如模型和消息。
    ///   可以使用 `ChatParam` 创建。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use openai4rs::*;
    /// use futures::StreamExt;
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let messages = vec![user!("Tell me a short story.")];
    ///     let request = ChatParam::new("Qwen/Qwen3-235B-A22B-Instruct-2507", &messages);
    ///     let mut stream = client.chat().create_stream(request).await?;
    ///
    ///     while let Some(chunk) = stream.next().await {
    ///         let chunk = chunk?;
    ///         if let Some(choice) = chunk.choices.first() {
    ///             if let Some(content) = &choice.delta.content {
    ///                 print!("{}", content);
    ///             }
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn create_stream(
        &self,
        param: ChatParam,
    ) -> Result<ReceiverStream<Result<ChatCompletionChunk, OpenAIError>>, OpenAIError> {
        let mut inner = param.take();
        inner
            .body
            .as_mut()
            .unwrap()
            .insert("stream".to_string(), serde_json::to_value(true).unwrap());

        let http_params = RequestSpec::new(
            |config| format!("{}/chat/completions", config.base_url()),
            move |config, request| {
                let mut builder = RequestBuilder::new(request);
                Self::apply_request_settings(&mut builder, inner);
                builder.bearer_auth(config.api_key());
                builder.take()
            },
        );
        self.http_client.post_json_sse(http_params).await
    }
}

impl Chat {
    fn apply_request_settings(builder: &mut RequestBuilder, params: InParam) {
        let body = params
            .body
            .unwrap_or_else(|| panic!("Unknown internal error, please submit an issue."));

        builder.body_fields(body);

        *builder.request_mut().headers_mut() = params.headers;

        if let Some(time) = params.extensions.get::<Timeout>() {
            builder.timeout(time.0);
        }

        if let Some(retry) = params.extensions.get::<RetryCount>() {
            builder.request_mut().extensions_mut().insert(retry.clone());
        }
    }
}
