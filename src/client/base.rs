use crate::modules::{
    chat::Chat, completions::Completions, embeddings::Embeddings, models::Models,
};
use crate::{config::Config, service::client::HttpClient};
use http::HeaderValue;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, RwLockReadGuard};

/// 用于与OpenAI兼容API交互的OpenAI客户端
///
/// 这是主要的客户端结构体，提供对聊天补全、文本补全和模型列表功能的访问。
/// 它使用async/await进行非阻塞操作并支持流式响应。
///
/// # 特性
///
/// - **聊天补全**: 支持流式和非流式的聊天补全
/// - **工具调用**: 支持在聊天补全中进行函数调用
/// - **推理模式**: 支持推理模型如qwq-32b
/// - **文本补全**: 支持传统的文本补全API
/// - **模型管理**: 列出和检索模型信息
/// - **线程安全**: 可以在线程间安全使用
///
/// # 示例
///
/// ## 基本用法
///
/// ```rust,no_run
/// use openai4rs::OpenAI;
/// use dotenvy::dotenv;
/// #[tokio::main]
/// async fn main() {
///     dotenv().ok();
///     let client = OpenAI::from_env().unwrap();
///
///     // 使用客户端进行各种操作
///     let models = client.models().list(openai4rs::ModelsParam::new()).await.unwrap();
///     println!("Available models: {:#?}", models);
/// }
/// ```
pub struct OpenAI {
    config: Arc<RwLock<Config>>,
    http_client: HttpClient,
    chat: Chat,
    completions: Completions,
    models: Models,
    embeddings: Embeddings,
}

impl OpenAI {
    /// 使用指定的API密钥和基础URL创建新的OpenAI客户端。
    ///
    /// # 参数
    ///
    /// * `api_key` - 您的OpenAI API密钥
    /// * `base_url` - API的基础URL (例如 "https://api.openai.com/v1")
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    ///
    /// let client = OpenAI::new("your-api-key", "https://api.openai.com/v1");
    /// ```
    #[must_use]
    pub fn new(api_key: &str, base_url: &str) -> OpenAI {
        let config = Config::new(api_key.to_string(), base_url.to_string());
        let http_client = HttpClient::new(config);

        OpenAI {
            chat: Chat::new(http_client.clone()),
            completions: Completions::new(http_client.clone()),
            models: Models::new(http_client.clone()),
            embeddings: Embeddings::new(http_client.clone()),
            config: http_client.config(),
            http_client,
        }
    }

    /// 使用自定义配置创建新的OpenAI客户端。
    ///
    /// 这允许您一次设置所有配置选项。
    ///
    /// # 参数
    ///
    /// * `config` - 自定义的`Config`实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::{Config, OpenAI};
    /// use openai4rs::header::HeaderValue;
    /// use std::time::Duration;
    ///
    /// let mut config = Config::new("your-api-key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.with_retry_count(5)
    ///       .with_timeout(Duration::from_secs(60))
    ///       .with_user_agent(HeaderValue::from_static("My-Custom-User-Agent"));
    ///
    /// let client = OpenAI::with_config(config);
    /// ```
    #[must_use]
    pub fn with_config(config: Config) -> OpenAI {
        let http_client = HttpClient::new(config);

        OpenAI {
            chat: Chat::new(http_client.clone()),
            completions: Completions::new(http_client.clone()),
            models: Models::new(http_client.clone()),
            embeddings: Embeddings::new(http_client.clone()),
            config: http_client.config(),
            http_client,
        }
    }

    #[doc = include_str!("../docs/from_env.md")]
    #[must_use]
    pub fn from_env() -> Result<Self, String> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| "The `OPENAI_API_KEY` environment variable is not set.")?;
        let base_url =
            std::env::var("OPENAI_BASE_URL").unwrap_or("https://api.openai.com/v1".to_string());

        let mut config = Config::new(api_key, base_url);

        // Read optional environment variables
        if let Ok(timeout) = std::env::var("OPENAI_TIMEOUT") {
            if let Ok(timeout) = timeout.parse::<u64>() {
                config.with_timeout(Duration::from_secs(timeout));
            }
        }

        if let Ok(connect_timeout) = std::env::var("OPENAI_CONNECT_TIMEOUT") {
            if let Ok(connect_timeout) = connect_timeout.parse::<u64>() {
                config.with_connect_timeout(Duration::from_secs(connect_timeout));
            }
        }

        if let Ok(retry_count) = std::env::var("OPENAI_RETRY_COUNT") {
            if let Ok(retry_count) = retry_count.parse::<usize>() {
                config.with_retry_count(retry_count);
            }
        }

        if let Ok(proxy) = std::env::var("OPENAI_PROXY") {
            config.with_proxy(proxy);
        }

        if let Ok(user_agent) = std::env::var("OPENAI_USER_AGENT") {
            config.with_user_agent(HeaderValue::from_str(&user_agent).unwrap_or_else(|_| {
                panic!("Cannot convert the value `{user_agent}` of environment variable `OPENAI_USER_AGENT` to HeaderValue, please check if the value is valid.")
            }));
        }

        Ok(Self::with_config(config))
    }
}

impl OpenAI {
    /// 更新客户端配置并重新创建HTTP客户端。
    ///
    /// 此方法允许您修改现有客户端的配置，并使用新设置自动重新创建内部HTTP客户端。
    ///
    /// # 参数
    ///
    /// * `update_fn` - 更新配置的函数
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    /// use std::time::Duration;
    /// #[tokio::main]
    /// async fn main() {
    /// let client = OpenAI::new("key", "https://api.openai.com/v1");
    ///
    /// // 一次更新多个设置
    /// client.update_config(|config| {
    ///     config.with_timeout(Duration::from_secs(60))
    ///           .with_retry_count(3)
    ///           .with_proxy("http://localhost:8080");
    /// }).await;
    /// }
    /// ```
    pub async fn update_config<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut Config),
    {
        {
            let mut config_guard = self.config.write().await;
            update_fn(&mut config_guard);
        }

        self.http_client.refresh_client().await;
    }

    #[doc = include_str!("../docs/chat.md")]
    #[inline]
    pub fn chat(&self) -> &Chat {
        &self.chat
    }

    #[doc = include_str!("../docs/completions.md")]
    #[inline]
    pub fn completions(&self) -> &Completions {
        &self.completions
    }

    #[doc = include_str!("../docs/models.md")]
    #[inline]
    pub fn models(&self) -> &Models {
        &self.models
    }

    #[doc = include_str!("../docs/embeddings.md")]
    #[inline]
    pub fn embeddings(&self) -> &Embeddings {
        &self.embeddings
    }

    /// 返回当前的基础URL。
    pub async fn base_url(&self) -> String {
        self.config.read().await.base_url().to_string()
    }

    /// 返回当前的API密钥。
    pub async fn api_key(&self) -> String {
        self.config.read().await.api_key().to_string()
    }

    /// 返回当前的配置。
    pub async fn config(&self) -> RwLockReadGuard<'_, Config> {
        self.config.read().await
    }

    /// 更新客户端的基础URL。
    ///
    /// 此操作不会重建HTTP客户端，因为它在每个请求中都会使用。
    pub async fn with_base_url(&self, base_url: impl Into<String>) {
        self.config.write().await.with_base_url(base_url);
    }

    /// 更新客户端的API密钥。
    ///
    /// 此操作不会重建HTTP客户端，因为API密钥在每个请求的头部中发送。
    pub async fn with_api_key(&self, api_key: impl Into<String>) {
        self.config.write().await.with_api_key(api_key);
    }

    /// 更新客户端的请求超时时间（以秒为单位）。
    ///
    /// 此操作将使用新设置重建内部HttpService。
    pub async fn with_timeout(&self, timeout: Duration) {
        self.update_config(|config| {
            config.with_timeout(timeout);
        })
        .await;
    }

    /// 更新客户端的连接超时时间（以秒为单位）。
    ///
    /// 此操作将使用新设置重建内部HttpService。
    pub async fn with_connect_timeout(&self, connect_timeout: Duration) {
        self.update_config(|config| {
            config.with_connect_timeout(connect_timeout);
        })
        .await;
    }

    /// 更新客户端的最大重试次数。
    ///
    /// 此操作不会重建HTTP客户端，因为它在每次重试时都会使用。
    pub async fn with_retry_count(&self, retry_count: usize) {
        self.config.write().await.with_retry_count(retry_count);
    }

    /// 更新客户端的HTTP代理。
    ///
    /// 此操作将使用新设置重建内部HttpService。
    pub async fn with_proxy(&self, proxy: impl Into<String>) {
        self.update_config(|config| {
            config.with_proxy(proxy);
        })
        .await;
    }

    /// 更新客户端的自定义用户代理。
    ///
    /// 此操作将使用新设置重建内部HttpService。
    pub async fn with_user_agent(&self, user_agent: HeaderValue) {
        self.update_config(|config| {
            config.with_user_agent(user_agent);
        })
        .await;
    }
}
