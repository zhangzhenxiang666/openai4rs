use super::params::{IntoRequestParams, RequestParams};
use super::types::{ChatCompletion, ChatCompletionChunk};
use crate::client::Config;
use crate::client::http::{openai_post_stream_with_lock, openai_post_with_lock};
use crate::error::OpenAIError;
use crate::utils::traits::ResponseHandler;
use reqwest::{Client, RequestBuilder};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio_stream::wrappers::ReceiverStream;

/// 处理聊天补全请求，包括流式和非流式模式。
pub struct Chat {
    config: Arc<RwLock<Config>>,
    client: Arc<RwLock<Client>>,
}

impl Chat {
    pub fn new(config: Arc<RwLock<Config>>, client: Arc<RwLock<Client>>) -> Self {
        Self { config, client }
    }

    /// 创建一个聊天补全。
    ///
    /// 此方法向 API 发送一个请求，并在单个响应中返回完整的补全。
    ///
    /// # 参数
    ///
    /// * `params` - 聊天补全的一组参数，例如模型和消息。可以使用 `chat_request` 创建。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use openai4rs::{OpenAI, chat_request, user};
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let messages = vec![user!("什么是 Rust？")];
    ///     let request = chat_request("deepseek/deepseek-chat-v3-0324:free", &messages);
    ///     let response = client.chat().create(request).await?;
    ///     println!("{:#?}", response);
    ///     Ok(())
    /// }
    /// ```
    pub async fn create<'a, T>(&self, params: T) -> Result<ChatCompletion, OpenAIError>
    where
        T: IntoRequestParams<'a>,
    {
        let mut params = params.into_request_params();
        params.stream = Some(false);

        let config = self.config.read().await;
        let retry_count = params
            .retry_count
            .unwrap_or_else(|| config.get_retry_count());

        let response = openai_post_with_lock(
            &self.client,
            "/chat/completions",
            |builder| Self::apply_request_settings(builder, &params),
            config.get_api_key(),
            config.get_base_url(),
            retry_count,
        )
        .await?;

        Self::process_unary(response).await
    }

    /// 创建一个流式聊天补全。
    ///
    /// 此方法返回一个 `ChatCompletionChunk` 事件流。这对于在生成补全时实时显示非常有用。
    ///
    /// # 参数
    ///
    /// * `params` - 聊天补全的一组参数，例如模型和消息。可以使用 `chat_request` 创建。
    ///
    /// # 示例
    ///
    /// ```rust,no_run
    /// use openai4rs::{OpenAI, chat_request, user};
    /// use futures::StreamExt;
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let messages = vec![user!("给我讲一个短篇故事。")];
    ///     let request = chat_request("deepseek/deepseek-chat-v3-0324:free", &messages);
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
    pub async fn create_stream<'a, T>(
        &self,
        params: T,
    ) -> Result<ReceiverStream<Result<ChatCompletionChunk, OpenAIError>>, OpenAIError>
    where
        T: IntoRequestParams<'a>,
    {
        let mut params = params.into_request_params();
        params.stream = Some(true);

        let config = self.config.read().await;
        let retry_count = params
            .retry_count
            .unwrap_or_else(|| config.get_retry_count());

        let event_source = openai_post_stream_with_lock(
            &self.client,
            "/chat/completions",
            |builder| Self::apply_request_settings(builder, &params),
            config.get_api_key(),
            config.get_base_url(),
            retry_count,
        )
        .await?;

        Ok(Self::process_stream(event_source).await)
    }
}

impl ResponseHandler for Chat {}

impl Chat {
    fn apply_request_settings(
        builder: RequestBuilder,
        params: &RequestParams<'_>,
    ) -> RequestBuilder {
        let mut builder = builder;

        if let Some(headers) = &params.extra_headers {
            for (k, v) in headers {
                builder = builder.header(k, v.to_string());
            }
        }

        if let Some(query) = &params.extra_query {
            builder = builder.query(query);
        }

        let mut body_map = HashMap::new();

        if let Ok(params_value) = serde_json::to_value(params) {
            if let Some(params_obj) = params_value.as_object() {
                body_map.extend(params_obj.iter().map(|(k, v)| (k.clone(), v.clone())));
            }
        }

        if let Some(extra_body) = &params.extra_body {
            body_map.extend(extra_body.iter().map(|(k, v)| (k.clone(), v.clone())));
        }

        builder = builder.json(&body_map);

        if let Some(timeout_seconds) = params.timeout_seconds {
            builder = builder.timeout(Duration::from_secs(timeout_seconds));
        }

        if let Some(user_agent) = &params.user_agent {
            builder = builder.header(reqwest::header::USER_AGENT, user_agent);
        }

        builder
    }
}
