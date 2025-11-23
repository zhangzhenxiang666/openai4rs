use crate::modules::{Chat, Completions, Embeddings, Models};
use crate::{config::Config, service::client::HttpClient};
use http::HeaderValue;
use std::time::Duration;

#[doc = include_str!("../docs/openai.md")]
pub struct OpenAI {
    http_client: HttpClient,
    chat: Chat,
    completions: Completions,
    models: Models,
    embeddings: Embeddings,
}

impl OpenAI {
    /// 根据api_key与base_url创建客户端
    ///
    /// 如果需要更精细的控制请使用`Config::builder()`来构建配置后并使用`build_openai`方法创建客户端。
    #[must_use]
    pub fn new(api_key: &str, base_url: &str) -> OpenAI {
        let config = Config::new(api_key.to_string(), base_url.to_string());
        let http_client = HttpClient::new(config);

        OpenAI {
            chat: Chat::new(http_client.clone()),
            completions: Completions::new(http_client.clone()),
            models: Models::new(http_client.clone()),
            embeddings: Embeddings::new(http_client.clone()),
            http_client,
        }
    }

    /// 根据配置创建客户端
    #[must_use]
    pub fn with_config(config: Config) -> OpenAI {
        let http_client = HttpClient::new(config);

        OpenAI {
            chat: Chat::new(http_client.clone()),
            completions: Completions::new(http_client.clone()),
            models: Models::new(http_client.clone()),
            embeddings: Embeddings::new(http_client.clone()),
            http_client,
        }
    }

    #[doc = include_str!("../docs/from_env.md")]
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

    #[inline]
    pub fn base_url(&self) -> String {
        self.http_client.config_read().base_url().to_string()
    }

    #[inline]
    pub fn api_key(&self) -> String {
        self.http_client.config_read().api_key().to_string()
    }

    #[inline]
    pub fn timeout(&self) -> Duration {
        self.http_client.config_read().timeout()
    }

    #[inline]
    pub fn connect_timeout(&self) -> Duration {
        self.http_client.config_read().connect_timeout()
    }

    #[inline]
    pub fn proxy(&self) -> Option<String> {
        self.http_client.config_read().proxy().cloned()
    }

    #[inline]
    pub fn user_agent(&self) -> Option<HeaderValue> {
        self.http_client.config_read().user_agent().cloned()
    }

    #[inline]
    pub fn retry_count(&self) -> usize {
        self.http_client.config_read().retry_count()
    }

    #[inline]
    pub fn with_base_url<T: Into<String>>(&self, base_url: T) {
        self.http_client.config_write().with_base_url(base_url);
    }

    #[inline]
    pub fn with_api_key<T: Into<String>>(&self, api_key: T) {
        self.http_client.config_write().with_api_key(api_key);
    }

    /// 更新客户端配置并重新创建HTTP客户端。
    ///
    /// 此方法允许您修改现有客户端的配置，并使用新设置自动重新创建内部HTTP客户端。
    ///
    /// # Example
    ///
    /// ```rust
    /// use openai4rs::OpenAI;
    /// use std::time::Duration;
    ///
    /// let client = OpenAI::new("key", "https://api.openai.com/v1");
    ///
    /// // 一次更新多个设置
    /// client.update_config(|config| {
    ///     config.with_timeout(Duration::from_secs(60))
    ///           .with_retry_count(3)
    ///           .with_proxy("http://localhost:8080");
    /// });
    /// ```
    pub fn update_config<F>(&self, update_fn: F)
    where
        F: FnOnce(&mut Config),
    {
        {
            let mut config_guard = self.http_client.config_write();
            update_fn(&mut config_guard);
        }

        self.http_client.refresh_client();
    }
}
