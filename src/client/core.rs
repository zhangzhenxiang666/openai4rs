use crate::{chat::Chat, completions::Completions, models::Models};
use derive_builder::Builder;
use reqwest::{Client, ClientBuilder, Proxy};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::sync::RwLock;

/// OpenAI 客户端配置
///
/// 包含用于连接到 OpenAI 兼容服务的 API 密钥、基础 URL 和 HTTP 请求设置。
///
/// # 示例
///
/// ```rust
/// use openai4rs::Config;
///
/// let config = Config::new(
///     "your-api-key".to_string(),
///     "https://api.openai.com/v1".to_string()
/// );
/// ```
#[derive(Builder)]
#[builder(name = "OpenAIConfigBuilder")]
#[builder(pattern = "owned")]
#[builder(setter(strip_option))]
pub struct Config {
    api_key: String,
    base_url: String,
    /// 请求失败的最大重试次数 默认值: 5
    #[builder(default = 5)]
    retry_count: u32,
    /// 请求超时时间（秒） 默认值: 60
    #[builder(default = 60)]
    timeout_seconds: u64,
    /// 连接超时时间（秒） 默认值: 10
    #[builder(default = 10)]
    connect_timeout_seconds: u64,
    /// HTTP 代理 URL (如有)
    #[builder(default = None)]
    proxy: Option<String>,
    /// 用户代理字符串
    #[builder(default = None)]
    user_agent: Option<String>,
}

impl OpenAIConfigBuilder {
    /// 构建配置并创建一个新的 OpenAI 客户端。
    ///
    /// 消费构建器以创建一个 [`Config`] 实例，然后用它来创建一个新的 [`OpenAI`] 客户端。
    /// 这是一个便捷方法，将构建配置和创建客户端合并为一步。
    ///
    /// # 错误
    ///
    /// 如果配置无效或无法构建，则返回错误。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let client = Config::builder()
    ///     .api_key("sk-your-api-key".to_string())
    ///     .base_url("https://api.openai.com/v1".to_string())
    ///     .retry_count(3)
    ///     .timeout_seconds(120)
    ///     .proxy("http://127.0.0.1:7890".to_string())
    ///     .user_agent("MyApp/1.0".to_string())
    ///     .build_openai()
    ///     .unwrap();
    /// ```
    pub fn build_openai(self) -> Result<OpenAI, OpenAIConfigBuilderError> {
        Ok(OpenAI::with_config(self.build()?))
    }
}

impl Config {
    /// 使用提供的 API 密钥和基础 URL 创建一个新配置。
    ///
    /// # 参数
    ///
    /// * `api_key` - 用于身份验证的 API 密钥。
    /// * `base_url` - API 端点的基础 URL。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let config = Config::new(
    ///     "sk-your-api-key".to_string(),
    ///     "https://api.openai.com/v1".to_string()
    /// );
    /// ```
    pub fn new(api_key: String, base_url: String) -> Self {
        Self::builder()
            .api_key(api_key)
            .base_url(base_url)
            .build()
            .unwrap()
    }

    /// 创建一个新的配置构建器。
    ///
    /// 返回一个新的 [`OpenAIConfigBuilder`] 实例，用于构造具有自定义设置的 [`Config`]。
    /// 这是创建具有非默认值的配置的首选方法。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let config = Config::builder()
    ///     .api_key("sk-your-api-key".to_string())
    ///     .base_url("https://api.openai.com/v1".to_string())
    ///     .retry_count(3)
    ///     .timeout_seconds(120)
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> OpenAIConfigBuilder {
        OpenAIConfigBuilder::create_empty()
    }
}

impl Config {
    /// 返回 API 密钥的副本。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let config = Config::new("test-key".to_string(), "https://api.openai.com/v1".to_string());
    /// assert_eq!(&config.get_api_key(), "test-key");
    /// ```
    pub fn get_api_key(&self) -> String {
        self.api_key.to_string()
    }

    /// 返回基础 URL 的副本。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let config = Config::new("test-key".to_string(), "https://api.openai.com/v1".to_string());
    /// assert_eq!(&config.get_base_url(), "https://api.openai.com/v1");
    /// ```
    pub fn get_base_url(&self) -> String {
        self.base_url.to_string()
    }

    /// 更新基础 URL。
    ///
    /// # 参数
    ///
    /// * `base_url` - 要设置的新基础 URL。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("test-key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_base_url("https://api.custom.com/v1".to_string());
    /// assert_eq!(config.get_base_url(), "https://api.custom.com/v1");
    /// ```
    pub fn set_base_url(&mut self, base_url: String) {
        self.base_url = base_url;
    }

    /// 更新 API 密钥。
    ///
    /// # 参数
    ///
    /// * `api_key` - 要设置的新 API 密钥。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("old-key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_api_key("new-key".to_string());
    /// assert_eq!(config.get_api_key(), "new-key");
    /// ```
    pub fn set_api_key(&mut self, api_key: String) {
        self.api_key = api_key;
    }

    /// 获取最大重试次数。
    pub fn get_retry_count(&self) -> u32 {
        self.retry_count
    }

    /// 设置最大重试次数。
    ///
    /// # 参数
    ///
    /// * `retry_count` - 重试失败请求的次数。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_retry_count(3);
    /// assert_eq!(config.get_retry_count(), 3);
    /// ```
    pub fn set_retry_count(&mut self, retry_count: u32) -> &mut Self {
        self.retry_count = retry_count;
        self
    }

    /// 获取请求超时时间（秒）。
    pub fn get_timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }

    /// 设置请求超时时间（秒）。
    ///
    /// # 参数
    ///
    /// * `timeout_seconds` - 请求的超时时间（秒）。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_timeout_seconds(30);
    /// assert_eq!(config.get_timeout_seconds(), 30);
    /// ```
    pub fn set_timeout_seconds(&mut self, timeout_seconds: u64) -> &mut Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// 获取连接超时时间（秒）。
    pub fn get_connect_timeout_seconds(&self) -> u64 {
        self.connect_timeout_seconds
    }

    /// 设置连接超时时间（秒）。
    ///
    /// # 参数
    ///
    /// * `connect_timeout_seconds` - 连接超时时间（秒）。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_connect_timeout_seconds(5);
    /// assert_eq!(config.get_connect_timeout_seconds(), 5);
    /// ```
    pub fn set_connect_timeout_seconds(&mut self, connect_timeout_seconds: u64) -> &mut Self {
        self.connect_timeout_seconds = connect_timeout_seconds;
        self
    }

    /// 获取代理 URL（如果已设置）。
    pub fn get_proxy(&self) -> Option<String> {
        self.proxy.clone()
    }

    /// 为请求设置 HTTP 代理。
    ///
    /// # 参数
    ///
    /// * `proxy` - 代理 URL (例如 "http://user:pass@host:port")。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_proxy(Some("http://localhost:8080".to_string()));
    /// assert_eq!(config.get_proxy(), Some("http://localhost:8080".to_string()));
    /// ```
    pub fn set_proxy(&mut self, proxy: Option<String>) -> &mut Self {
        self.proxy = proxy;
        self
    }

    /// 获取用户代理字符串（如果已设置）。
    pub fn get_user_agent(&self) -> Option<String> {
        self.user_agent.clone()
    }

    /// 设置自定义用户代理字符串。
    ///
    /// # 参数
    ///
    /// * `user_agent` - 自定义用户代理字符串。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::Config;
    ///
    /// let mut config = Config::new("key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_user_agent(Some("MyApp/1.0".to_string()));
    /// assert_eq!(config.get_user_agent(), Some("MyApp/1.0".to_string()));
    /// ```
    pub fn set_user_agent(&mut self, user_agent: Option<String>) -> &mut Self {
        self.user_agent = user_agent;
        self
    }

    /// 使用配置的设置构建一个 `reqwest::Client`。
    pub fn build_client(&self) -> Client {
        let mut client_builder = ClientBuilder::new()
            .timeout(Duration::from_secs(self.timeout_seconds))
            .connect_timeout(Duration::from_secs(self.connect_timeout_seconds));

        // 如果配置了代理，则添加
        if let Some(proxy_url) = &self.proxy {
            if let Ok(proxy) = Proxy::all(proxy_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }

        // 如果配置了用户代理，则添加
        if let Some(user_agent) = &self.user_agent {
            client_builder = client_builder.user_agent(user_agent);
        }

        client_builder.build().unwrap_or_else(|_| Client::new())
    }
}

/// 用于与 OpenAI 兼容 API 交互的 OpenAI 客户端
///
/// 这是主客户端结构体，提供对聊天补全、文本补全和模型列出功能的访问。
/// 它使用 async/await 进行非阻塞操作，并支持流式响应。
///
/// # 特性
///
/// - **聊天补全**: 同时支持流式和非流式聊天补全
/// - **工具调用**: 支持聊天补全中的函数调用
/// - **推理模式**: 支持像 qwq-32b 这样的推理模型
/// - **文本补全**: 支持旧版文本补全 API
/// - **模型管理**: 列出和检索模型信息
/// - **线程安全**: 可在多个线程间安全使用
///
/// # 示例
///
/// ## 基本用法
///
/// ```rust
/// use openai4rs::OpenAI;
/// use dotenvy::dotenv;
/// #[tokio::main]
/// async fn main() {
///     dotenv().ok();
///     let client = OpenAI::from_env().unwrap();
///     
///     // 使用客户端进行各种操作
///     let models = client.models().list(openai4rs::models_request()).await.unwrap();
///     println!("可用模型: {:#?}", models);
/// }
/// ```
pub struct OpenAI {
    config: Arc<RwLock<Config>>,
    client: Arc<RwLock<Client>>,
    chat: OnceLock<Chat>,
    completions: OnceLock<Completions>,
    models: OnceLock<Models>,
}

impl OpenAI {
    /// 使用指定的 API 密钥和基础 URL 创建一个新的 OpenAI 客户端。
    ///
    /// # 参数
    ///
    /// * `api_key` - 您的 OpenAI API 密钥
    /// * `base_url` - API 的基础 URL (例如 "https://api.openai.com/v1")
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    ///
    /// let client = OpenAI::new("your-api-key", "https://api.openai.com/v1");
    /// ```
    pub fn new(api_key: &str, base_url: &str) -> Self {
        let config = Config::new(api_key.to_string(), base_url.to_string());
        let client = config.build_client();

        Self {
            config: Arc::new(RwLock::new(config)),
            client: Arc::new(RwLock::new(client)),
            chat: OnceLock::new(),
            completions: OnceLock::new(),
            models: OnceLock::new(),
        }
    }

    /// 使用自定义配置创建一个新的 OpenAI 客户端。
    ///
    /// 这允许您一次性设置所有配置选项。
    ///
    /// # 参数
    ///
    /// * `config` - 一个自定义的 `Config` 实例
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::{Config, OpenAI};
    ///
    /// let mut config = Config::new("your-api-key".to_string(), "https://api.openai.com/v1".to_string());
    /// config.set_retry_count(3)
    ///       .set_timeout_seconds(120)
    ///       .set_user_agent(Some("MyApp/1.0".to_string()));
    ///
    /// let client = OpenAI::with_config(config);
    /// ```
    pub fn with_config(config: Config) -> Self {
        let client = config.build_client();

        Self {
            config: Arc::new(RwLock::new(config)),
            client: Arc::new(RwLock::new(client)),
            chat: OnceLock::new(),
            completions: OnceLock::new(),
            models: OnceLock::new(),
        }
    }

    /// 更新客户端配置并重新创建 HTTP 客户端。
    ///
    /// 此方法允许您修改现有客户端的配置，并使用新设置自动重新创建内部 HTTP 客户端。
    ///
    /// # 参数
    ///
    /// * `update_fn` - 一个更新配置的函数
    ///
    /// # 示例
    ///
    /// ```rust
    /// # use openai4rs::OpenAI;
    /// # #[tokio::main]
    /// # async fn main() {
    /// let client = OpenAI::new("key", "https://api.openai.com/v1");
    ///
    /// // 一次性更新多个设置
    /// client.update_config(|config| {
    ///     config.set_timeout_seconds(120)
    ///           .set_retry_count(3)
    ///           .set_proxy(Some("http://localhost:8080".to_string()));
    /// }).await;
    /// # }
    /// ```
    pub async fn update_config<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut Config),
    {
        let new_client = {
            // 更新配置
            let mut config_guard = self.config.write().await;
            update_fn(&mut config_guard);

            // 使用新设置重新创建 HTTP 客户端
            config_guard.build_client()
        };

        // 更新客户端
        let mut client_guard = self.client.write().await;
        *client_guard = new_client;
    }

    /// 从环境变量创建一个新的 OpenAI 客户端。
    ///
    /// 查找以下环境变量：
    /// - `OPENAI_API_KEY` (必需): 您的 API 密钥
    /// - `OPENAI_BASE_URL` (可选): 基础 URL, 默认为 "https://api.openai.com/v1"
    /// - `OPENAI_TIMEOUT` (可选): 请求超时秒数, 默认为 60
    /// - `OPENAI_CONNECT_TIMEOUT` (可选): 连接超时秒数, 默认为 10
    /// - `OPENAI_RETRY_COUNT` (可选): 重试次数, 默认为 5
    /// - `OPENAI_PROXY` (可选): HTTP 代理 URL
    /// - `OPENAI_USER_AGENT` (可选): 自定义用户代理字符串
    ///
    /// # 错误
    ///
    /// 如果环境中未设置 `OPENAI_API_KEY`，则返回错误。
    ///
    /// # 示例
    ///
    /// ```bash
    /// # 设置环境变量
    /// export OPENAI_API_KEY="sk-your-api-key"
    /// export OPENAI_BASE_URL="https://api.openai.com/v1"  # 可选
    /// export OPENAI_TIMEOUT="120"  # 可选, 120 秒
    /// export OPENAI_RETRY_COUNT="3"  # 可选, 重试 3 次
    /// ```
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), String> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     
    ///     // 客户端已准备就绪
    ///     println!("已连接到: {}", client.get_base_url().await);
    ///     Ok(())
    /// }
    /// ```
    pub fn from_env() -> Result<Self, String> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| "OPENAI_API_KEY not set")?;
        let base_url =
            std::env::var("OPENAI_BASE_URL").unwrap_or("https://api.openai.com/v1".to_string());

        let mut config = Config::new(api_key, base_url);

        // 读取可选的环境变量
        if let Ok(timeout) = std::env::var("OPENAI_TIMEOUT") {
            if let Ok(timeout) = timeout.parse::<u64>() {
                config.set_timeout_seconds(timeout);
            }
        }

        if let Ok(connect_timeout) = std::env::var("OPENAI_CONNECT_TIMEOUT") {
            if let Ok(connect_timeout) = connect_timeout.parse::<u64>() {
                config.set_connect_timeout_seconds(connect_timeout);
            }
        }

        if let Ok(retry_count) = std::env::var("OPENAI_RETRY_COUNT") {
            if let Ok(retry_count) = retry_count.parse::<u32>() {
                config.set_retry_count(retry_count);
            }
        }

        if let Ok(proxy) = std::env::var("OPENAI_PROXY") {
            config.set_proxy(Some(proxy));
        }

        if let Ok(user_agent) = std::env::var("OPENAI_USER_AGENT") {
            config.set_user_agent(Some(user_agent));
        }

        Ok(Self::with_config(config))
    }
}

impl OpenAI {
    /// 返回对聊天补全客户端的引用。
    ///
    /// 使用此客户端执行聊天补全，包括流式响应、工具调用和推理模式交互。
    ///
    /// # 示例
    ///
    /// ## 基本聊天补全
    ///
    /// ```rust
    /// use openai4rs::{OpenAI, chat_request, user};
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let messages = vec![user!("你好，你好吗？")];
    ///
    ///     let response = client
    ///                     .chat()
    ///                     .create(chat_request("deepseek/deepseek-chat-v3-0324:free", &messages))
    ///                     .await?;
    ///
    ///     println!("响应: {:#?}", response);
    ///     Ok(())
    ///  }
    /// ```
    ///
    /// ## 流式聊天补全
    ///
    /// ```rust
    /// use futures::StreamExt;
    /// use openai4rs::{OpenAI, chat_request, user};
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let messages = vec![user!("给我讲个故事")];
    ///
    ///     let mut stream = client.chat().create_stream(chat_request("deepseek/deepseek-chat-v3-0324:free", &messages).max_completion_tokens(64)).await?;
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
    pub fn chat(&self) -> &Chat {
        self.chat
            .get_or_init(|| Chat::new(Arc::clone(&self.config), Arc::clone(&self.client)))
    }

    /// 返回对补全客户端的引用。
    ///
    /// 用于旧版文本补全（非聊天格式）。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::{OpenAI, completions_request};
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     let response = client
    ///         .completions()
    ///         .create(completions_request("deepseek/deepseek-chat-v3-0324:free", "写一首关于 Rust 编程语言的诗").max_tokens(64))
    ///         .await?;
    ///
    ///     println!("响应: {:#?}", response);
    ///     Ok(())
    /// }
    /// ```
    pub fn completions(&self) -> &Completions {
        self.completions
            .get_or_init(|| Completions::new(Arc::clone(&self.config), Arc::clone(&self.client)))
    }

    /// 返回对模型客户端的引用。
    ///
    /// 用于列出可用模型或检索模型信息。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::{OpenAI, models_request};
    /// use dotenvy::dotenv;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     dotenv().ok();
    ///     let client = OpenAI::from_env()?;
    ///     // 列出所有可用模型
    ///     let models = client
    ///         .models()
    ///         .list(models_request())
    ///         .await?;
    ///
    ///     for model in models.data {
    ///         println!("模型: {}", model.id);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn models(&self) -> &Models {
        self.models
            .get_or_init(|| Models::new(Arc::clone(&self.config), Arc::clone(&self.client)))
    }

    /// 返回当前的基础 URL。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    /// #[tokio::main]
    /// async fn main(){
    ///     let client = OpenAI::new("key", "https://api.openai.com/v1");
    ///     assert_eq!(client.get_base_url().await, "https://api.openai.com/v1");
    /// }
    /// ```
    pub async fn get_base_url(&self) -> String {
        self.config.read().await.get_base_url()
    }

    /// 返回当前的 API 密钥。
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = OpenAI::new("test-key", "https://api.openai.com/v1");
    ///     assert_eq!(client.get_api_key().await, "test-key");
    /// }
    /// ```
    pub async fn get_api_key(&self) -> String {
        self.config.read().await.get_api_key()
    }

    /// 更新客户端的基础 URL。
    ///
    /// 这对于在不同的 API 端点之间切换或从一个服务迁移到另一个服务时非常有用。
    ///
    /// # 参数
    ///
    /// * `base_url` - 要使用的新基础 URL
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = OpenAI::new("key", "https://api.openai.com/v1");
    ///
    ///     // 切换到本地服务器
    ///     client.set_base_url("http://localhost:8000/v1".to_string()).await;
    ///     assert_eq!(client.get_base_url().await, "http://localhost:8000/v1");
    ///
    ///     // 切换到 Azure OpenAI
    ///     client.set_base_url("https://your-resource.openai.azure.com/openai/deployments/your-deployment".to_string()).await;
    /// }
    /// ```
    pub async fn set_base_url(&self, base_url: String) {
        self.config.write().await.set_base_url(base_url);
    }

    /// 更新客户端的 API 密钥。
    ///
    /// 这对于密钥轮换或在不同 API 帐户之间切换时非常有用。
    ///
    /// # 参数
    ///
    /// * `api_key` - 要使用的新 API 密钥
    ///
    /// # 示例
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    /// #[tokio::main]
    /// async fn main() {
    ///     let client = OpenAI::new("old-key", "https://api.openai.com/v1");
    ///
    ///     // 轮换到新密钥
    ///     client.set_api_key("new-key".to_string()).await;
    ///     assert_eq!(client.get_api_key().await, "new-key");
    /// }
    /// ```
    pub async fn set_api_key(&self, api_key: String) {
        self.config.write().await.set_api_key(api_key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chat::*, completions_request, models_request, user};
    use dotenvy::dotenv;
    const MODEL_NAME: &str = "deepseek/deepseek-chat-v3-0324:free";

    #[test]
    fn test_config_builder() {
        let config = Config::builder()
            .api_key("test-key".to_string())
            .base_url("https://api.test.com/v1".to_string())
            .retry_count(3)
            .timeout_seconds(120)
            .connect_timeout_seconds(15)
            .proxy("http://proxy.test.com:8080".to_string())
            .user_agent("TestAgent/1.0".to_string())
            .build()
            .unwrap();

        assert_eq!(config.get_api_key(), "test-key");
        assert_eq!(config.get_base_url(), "https://api.test.com/v1");
        assert_eq!(config.get_retry_count(), 3);
        assert_eq!(config.get_timeout_seconds(), 120);
        assert_eq!(config.get_connect_timeout_seconds(), 15);
        assert_eq!(
            config.get_proxy(),
            Some("http://proxy.test.com:8080".to_string())
        );
        assert_eq!(config.get_user_agent(), Some("TestAgent/1.0".to_string()));
    }

    #[test]
    fn test_config_builder_defaults() {
        let config = Config::builder()
            .api_key("test-key".to_string())
            .base_url("https://api.test.com/v1".to_string())
            .build()
            .unwrap();

        assert_eq!(config.get_retry_count(), 5); // default value
        assert_eq!(config.get_timeout_seconds(), 60); // default value
        assert_eq!(config.get_connect_timeout_seconds(), 10); // default value
        assert_eq!(config.get_proxy(), None); // default value
        assert_eq!(config.get_user_agent(), None); // default value
    }

    #[tokio::test]
    async fn test_build_openai() {
        let client = Config::builder()
            .api_key("test-key".to_string())
            .base_url("https://api.test.com/v1".to_string())
            .build_openai()
            .unwrap();

        let config = client.config.read().await;

        assert_eq!(config.get_api_key(), "test-key");
        assert_eq!(config.get_base_url(), "https://api.test.com/v1");
    }

    #[test]
    fn test_config_new() {
        let config = Config::new(
            "test-key".to_string(),
            "https://api.test.com/v1".to_string(),
        );

        assert_eq!(config.get_api_key(), "test-key");
        assert_eq!(config.get_base_url(), "https://api.test.com/v1");
    }

    #[test]
    fn test_config_setters() {
        let mut config = Config::new("old-key".to_string(), "https://old-api.com/v1".to_string());

        config.set_api_key("new-key".to_string());
        config.set_base_url("https://new-api.com/v1".to_string());
        config.set_retry_count(2);
        config.set_timeout_seconds(30);
        config.set_connect_timeout_seconds(5);
        config.set_proxy(Some("http://proxy.example.com:8080".to_string()));
        config.set_user_agent(Some("CustomAgent/2.0".to_string()));

        assert_eq!(config.get_api_key(), "new-key");
        assert_eq!(config.get_base_url(), "https://new-api.com/v1");
        assert_eq!(config.get_retry_count(), 2);
        assert_eq!(config.get_timeout_seconds(), 30);
        assert_eq!(config.get_connect_timeout_seconds(), 5);
        assert_eq!(
            config.get_proxy(),
            Some("http://proxy.example.com:8080".to_string())
        );
        assert_eq!(config.get_user_agent(), Some("CustomAgent/2.0".to_string()));
    }

    #[tokio::test]
    async fn test_chat() {
        dotenv().ok();
        let client = OpenAI::from_env().unwrap();
        let messages = vec![user!("Hello")];

        let mut retries = 3;
        while retries > 0 {
            let request = chat_request(MODEL_NAME, &messages).temperature(0.0);
            match client.chat().create(request).await {
                Ok(result) => {
                    assert_eq!(
                        Some("Hello! 😊 How can I assist you today?".into()),
                        result.choices[0].message.content
                    );
                    return;
                }
                Err(e) if e.is_retryable() => {
                    retries -= 1;
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
                Err(e) => {
                    panic!("Non-retryable error: {}", e);
                }
            }
        }
        panic!("Test failed after multiple retries");
    }

    #[tokio::test]
    async fn test_openai_error_authentication() {
        let base_url = "https://openrouter.ai/api/v1";
        let api_key = "******";
        let client = OpenAI::new(api_key, base_url);
        let messages = vec![user!("Hello")];
        let result = client
            .chat()
            .create(
                chat_request(MODEL_NAME, &messages)
                    .temperature(0.0)
                    .max_completion_tokens(512),
            )
            .await;
        match result {
            Ok(_) => panic!("Unexpected success response"),
            Err(err) => {
                if !err.is_authentication() {
                    panic!("Unexpected error: {:?}", err);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_models_list() {
        dotenv().ok();
        let client = OpenAI::from_env().unwrap();
        let models = client.models().list(models_request()).await;
        assert!(models.is_ok())
    }

    #[tokio::test]
    async fn test_completions() {
        dotenv().ok();
        let client = OpenAI::from_env().unwrap();

        let mut retries = 3;
        while retries > 0 {
            let request = completions_request(MODEL_NAME, "Hello")
                .temperature(0.0)
                .max_tokens(100);
            match client.completions().create(request).await {
                Ok(_) => {
                    // If the request succeeds, we can break the loop.
                    return;
                }
                Err(e) if e.is_retryable() => {
                    retries -= 1;
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
                Err(e) => {
                    panic!("Non-retryable error: {}", e);
                }
            }
        }
        panic!("Test failed after multiple retries");
    }
}
