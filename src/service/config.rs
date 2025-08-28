use derive_builder::Builder;

/// HTTP client configuration for connecting to an API service.
///
/// This struct holds settings related to the underlying HTTP transport layer,
/// such as timeouts, proxy settings, and user agent. It is designed to be
/// reusable and independent of any specific API's business logic.
///
/// The configuration uses the builder pattern for flexible construction, allowing
/// users to set only the options they need while using sensible defaults for others.
#[derive(Debug, Clone, Builder)]
#[builder(name = "HttpConfigBuilder", pattern = "owned", setter(strip_option))]
pub struct HttpConfig {
    /// Request timeout in seconds. Default: 300
    ///
    /// This is the total time allowed for a request to complete, including
    /// DNS resolution, connection establishment, sending the request,
    /// and receiving the response.
    #[builder(default = 300)]
    pub timeout_seconds: u64,

    /// Connection timeout in seconds. Default: 10
    ///
    /// This is the maximum time allowed to establish a connection to the server.
    /// It is a subset of the overall request timeout.
    #[builder(default = 10)]
    pub connect_timeout_seconds: u64,

    /// HTTP proxy URL (if any)
    ///
    /// If set, all HTTP requests will be routed through this proxy server.
    /// Supported proxy schemes include HTTP, HTTPS, and SOCKS.
    #[builder(default = None)]
    pub proxy: Option<String>,

    /// User agent string
    ///
    /// If set, this value will be used as the User-Agent header for all requests.
    /// If not set, the default reqwest User-Agent will be used.
    #[builder(default = None)]
    pub user_agent: Option<String>,
    // NOTE: pool_idle_timeout and other reqwest-specific settings can be added here in the future.
}

impl HttpConfig {
    /// Creates a new configuration builder.
    ///
    /// This is the preferred way to construct an HttpConfig, allowing for
    /// flexible configuration with sensible defaults.
    ///
    /// # Examples
    ///
    /// ```
    /// use openai4rs::service::config::HttpConfig;
    ///
    /// let config = HttpConfig::builder()
    ///     .timeout_seconds(60)
    ///     .proxy("http://proxy.example.com:8080".to_string())
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> HttpConfigBuilder {
        HttpConfigBuilder::default()
    }
}

impl Default for HttpConfig {
    /// Returns the default HTTP configuration.
    ///
    /// The default configuration includes:
    /// - 300 second request timeout
    /// - 10 second connection timeout
    /// - No proxy
    /// - No custom user agent
    fn default() -> Self {
        Self {
            timeout_seconds: 300,
            connect_timeout_seconds: 10,
            proxy: None,
            user_agent: None,
        }
    }
}
