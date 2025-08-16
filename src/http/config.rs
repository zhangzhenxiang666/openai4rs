use derive_builder::Builder;

/// HTTP client configuration for connecting to an API service.
///
/// This struct holds settings related to the underlying HTTP transport layer,
/// such as timeouts, proxy settings, and user agent. It is designed to be
/// reusable and independent of any specific API's business logic.
#[derive(Debug, Clone, Builder)]
#[builder(name = "HttpConfigBuilder", pattern = "owned", setter(strip_option))]
pub struct HttpConfig {
    /// Request timeout in seconds. Default: 300
    #[builder(default = 300)]
    pub timeout_seconds: u64,

    /// Connection timeout in seconds. Default: 10
    #[builder(default = 10)]
    pub connect_timeout_seconds: u64,

    /// HTTP proxy URL (if any)
    #[builder(default = None)]
    pub proxy: Option<String>,

    /// User agent string
    #[builder(default = None)]
    pub user_agent: Option<String>,
    // NOTE: pool_idle_timeout and other reqwest-specific settings can be added here in the future.
}

impl HttpConfig {
    /// Creates a new configuration builder.
    pub fn builder() -> HttpConfigBuilder {
        HttpConfigBuilder::default()
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 300,
            connect_timeout_seconds: 10,
            proxy: None,
            user_agent: None,
        }
    }
}
