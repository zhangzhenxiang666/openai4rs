use super::{
    base::{BaseConfig, BaseConfigBuilder},
    http::{HttpConfig, HttpConfigBuilder},
};
use crate::OpenAI;
use std::{collections::HashMap, fmt};

#[derive(Debug)]
pub enum ConfigBuildError {
    /// Required fields missing error
    RequiredFieldMissing(String),
    /// Validation error
    ValidationError(String),
}

impl fmt::Display for ConfigBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigBuildError::RequiredFieldMissing(field) => {
                write!(f, "Required field missing: {}", field)
            }
            ConfigBuildError::ValidationError(msg) => {
                write!(f, "Validation error: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConfigBuildError {}

// Implement From trait to adapt builder-generated error types
impl From<super::http::HttpConfigBuilderError> for ConfigBuildError {
    fn from(err: super::http::HttpConfigBuilderError) -> Self {
        ConfigBuildError::RequiredFieldMissing(err.to_string())
    }
}

impl From<super::base::BaseConfigBuilderError> for ConfigBuildError {
    fn from(err: super::base::BaseConfigBuilderError) -> Self {
        ConfigBuildError::RequiredFieldMissing(err.to_string())
    }
}

/// Main configuration struct containing all settings for API communication
pub struct Config {
    /// Base configuration containing API key and URL
    base: BaseConfig,
    /// HTTP-specific configuration (timeouts, proxy, etc.)
    http: HttpConfig,
    /// Number of retry attempts for failed requests
    retry_count: u32,
}
impl Config {
    /// Creates a new Config with the specified API key and base URL
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for authentication
    /// * `base_url` - The base URL for API requests
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            base: BaseConfig::new(api_key.into(), base_url.into()),
            http: HttpConfig::default(),
            retry_count: 5,
        }
    }

    /// Creates a new ConfigBuilder for fluent configuration
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder {
            retry_count: 5,
            base_builder: BaseConfigBuilder::default(),
            http_builder: HttpConfigBuilder::default(),
        }
    }

    /// Returns the API key
    #[inline]
    pub fn api_key(&self) -> &str {
        self.base.api_key()
    }

    /// Returns the base URL
    #[inline]
    pub fn base_url(&self) -> &str {
        self.base.base_url()
    }

    /// Returns the retry count
    #[inline]
    pub fn retry_count(&self) -> u32 {
        self.retry_count
    }

    /// Returns the request timeout in seconds
    #[inline]
    pub fn timeout_seconds(&self) -> u64 {
        self.http.timeout_seconds()
    }

    /// Returns an optional proxy URL
    #[inline]
    pub fn proxy(&self) -> Option<&String> {
        self.http.proxy()
    }

    /// Returns an optional custom user agent string
    #[inline]
    pub fn user_agent(&self) -> Option<&String> {
        self.http.user_agent()
    }

    /// Returns the connection timeout in seconds
    #[inline]
    pub fn connect_timeout_seconds(&self) -> u64 {
        self.http.connect_timeout_seconds()
    }

    /// Returns a reference to the HTTP configuration
    #[inline]
    pub fn http(&self) -> &HttpConfig {
        &self.http
    }

    /// Returns a reference to the base configuration
    #[inline]
    pub fn base(&self) -> &super::base::BaseConfig {
        &self.base
    }

    /// Sets a new base URL for this configuration
    ///
    /// # Arguments
    ///
    /// * `base_url` - The new base URL to use
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn with_base_url(&mut self, base_url: impl Into<String>) -> &mut Self {
        self.base.with_base_url(base_url);
        self
    }

    /// Sets a new API key for this configuration
    ///
    /// # Arguments
    ///
    /// * `api_key` - The new API key to use
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn with_api_key(&mut self, api_key: impl Into<String>) -> &mut Self {
        self.base.with_api_key(api_key);
        self
    }

    /// Sets the number of retry attempts for failed requests
    ///
    /// # Arguments
    ///
    /// * `retry_count` - The number of retry attempts
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn with_retry_count(&mut self, retry_count: u32) -> &mut Self {
        self.retry_count = retry_count;
        self
    }

    /// Sets the request timeout in seconds
    ///
    /// # Arguments
    ///
    /// * `timeout_seconds` - The timeout value in seconds
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn with_timeout_seconds(&mut self, timeout_seconds: u64) -> &mut Self {
        self.http.with_timeout_seconds(timeout_seconds);
        self
    }

    /// Sets the connection timeout in seconds
    ///
    /// # Arguments
    ///
    /// * `connect_timeout_seconds` - The connection timeout value in seconds
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn with_connect_timeout_seconds(&mut self, connect_timeout_seconds: u64) -> &mut Self {
        self.http
            .with_connect_timeout_seconds(connect_timeout_seconds);
        self
    }

    /// Sets an HTTP proxy for requests
    ///
    /// # Arguments
    ///
    /// * `proxy` - The proxy URL to use
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn with_proxy(&mut self, proxy: impl Into<String>) -> &mut Self {
        self.http.with_proxy(proxy);
        self
    }

    /// Sets a custom user agent string
    ///
    /// # Arguments
    ///
    /// * `user_agent` - The user agent string to use
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn with_user_agent(&mut self, user_agent: impl Into<String>) -> &mut Self {
        self.http.with_user_agent(user_agent);
        self
    }
}

/// Builder for creating Config instances with fluent API
pub struct ConfigBuilder {
    /// Number of retry attempts for failed requests
    retry_count: u32,
    /// Builder for BaseConfig
    base_builder: BaseConfigBuilder,
    /// Builder for HttpConfig
    http_builder: HttpConfigBuilder,
}

impl ConfigBuilder {
    /// Builds the Config instance from the current builder state
    ///
    /// # Returns
    ///
    /// A Result containing either the Config instance or a ConfigBuildError
    pub fn build(self) -> Result<Config, ConfigBuildError> {
        Ok(Config {
            base: self.base_builder.build()?,
            http: self.http_builder.build()?,
            retry_count: self.retry_count,
        })
    }

    /// Builds an OpenAI client instance from the current configuration
    ///
    /// # Returns
    ///
    /// A Result containing either the OpenAI client instance or a ConfigBuildError
    pub fn build_openai(self) -> Result<OpenAI, ConfigBuildError> {
        Ok(OpenAI::with_config(self.build()?))
    }

    /// Sets the API key for the configuration
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key to use
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.base_builder = self.base_builder.api_key(api_key.into());
        self
    }

    /// Sets the base URL for the configuration
    ///
    /// # Arguments
    ///
    /// * `base_url` - The base URL to use
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_builder = self.base_builder.base_url(base_url.into());
        self
    }

    /// Sets the retry count for the configuration
    ///
    /// # Arguments
    ///
    /// * `retry_count` - The number of retry attempts
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn retry_count(mut self, retry_count: u32) -> Self {
        self.retry_count = retry_count;
        self
    }

    /// Sets the request timeout in seconds for the configuration
    ///
    /// # Arguments
    ///
    /// * `timeout_seconds` - The timeout value in seconds
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn timeout_seconds(mut self, timeout_seconds: u64) -> Self {
        self.http_builder = self.http_builder.timeout_seconds(timeout_seconds);
        self
    }

    /// Sets the connection timeout in seconds for the configuration
    ///
    /// # Arguments
    ///
    /// * `connect_timeout_seconds` - The connection timeout value in seconds
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn connect_timeout_seconds(mut self, connect_timeout_seconds: u64) -> Self {
        self.http_builder = self
            .http_builder
            .connect_timeout_seconds(connect_timeout_seconds);
        self
    }

    /// Sets an HTTP proxy for the configuration
    ///
    /// # Arguments
    ///
    /// * `proxy` - The proxy URL to use
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn proxy(mut self, proxy: impl Into<String>) -> Self {
        self.http_builder = self.http_builder.proxy(proxy.into());
        self
    }

    /// Sets a custom user agent string for the configuration
    ///
    /// # Arguments
    ///
    /// * `user_agent` - The user agent string to use
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.http_builder = self.http_builder.user_agent(user_agent.into());
        self
    }

    /// Adds a global header to the HTTP configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The header name
    /// * `value` - The header value
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.http_builder = self.http_builder.header(key.into(), value.into());
        self
    }

    /// Adds a global query parameter to the HTTP configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The query parameter name
    /// * `value` - The query parameter value
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn query(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.http_builder = self.http_builder.query(key.into(), value.into());
        self
    }

    /// Adds a global body field to the HTTP configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The body field name
    /// * `value` - The body field value
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn body(mut self, key: impl Into<String>, value: impl Into<serde_json::Value>) -> Self {
        self.http_builder = self.http_builder.body(key.into(), value.into());
        self
    }

    /// Sets multiple global headers in the HTTP configuration.
    ///
    /// # Arguments
    ///
    /// * `headers` - A map of header names to values
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.http_builder = self.http_builder.headers(headers);
        self
    }

    /// Sets multiple global query parameters in the HTTP configuration.
    ///
    /// # Arguments
    ///
    /// * `queries` - A map of query parameter names to values
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn querys(mut self, queries: HashMap<String, String>) -> Self {
        self.http_builder = self.http_builder.querys(queries);
        self
    }

    /// Sets multiple global body fields in the HTTP configuration.
    ///
    /// # Arguments
    ///
    /// * `bodys` - A map of body field names to values
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining
    pub fn bodys(mut self, bodys: HashMap<String, serde_json::Value>) -> Self {
        self.http_builder = self.http_builder.bodys(bodys);
        self
    }
}
