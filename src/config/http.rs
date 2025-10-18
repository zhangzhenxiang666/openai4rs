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
    timeout_seconds: u64,

    /// Connection timeout in seconds. Default: 10
    ///
    /// This is the maximum time allowed to establish a connection to the server.
    /// It is a subset of the overall request timeout.
    #[builder(default = 10)]
    connect_timeout_seconds: u64,

    /// HTTP proxy URL (if any)
    ///
    /// If set, all HTTP requests will be routed through this proxy server.
    /// Supported proxy schemes include HTTP, HTTPS, and SOCKS.
    #[builder(default = None)]
    proxy: Option<String>,

    /// User agent string
    ///
    /// If set, this value will be used as the User-Agent header for all requests.
    /// If not set, the default reqwest User-Agent will be used.
    #[builder(default = None)]
    user_agent: Option<String>,
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
    /// use openai4rsc::HttpConfig;
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

    /// Returns the request timeout in seconds.
    ///
    /// This value determines the total time allowed for a request to complete,
    /// including DNS resolution, connection establishment, sending the request,
    /// and receiving the response.
    #[inline]
    pub fn timeout_seconds(&self) -> u64 {
        self.timeout_seconds
    }

    /// Returns the connection timeout in seconds.
    ///
    /// This value determines the maximum time allowed to establish a connection to the server.
    /// It is a subset of the overall request timeout.
    #[inline]
    pub fn connect_timeout_seconds(&self) -> u64 {
        self.connect_timeout_seconds
    }

    /// Returns an optional reference to the proxy URL.
    ///
    /// If a proxy is configured, this method returns Some containing a reference to the proxy URL.
    /// Otherwise, it returns None.
    #[inline]
    pub fn proxy(&self) -> Option<&String> {
        self.proxy.as_ref()
    }

    /// Returns an optional reference to the user agent string.
    ///
    /// If a custom user agent is configured, this method returns Some containing a reference to the user agent string.
    /// Otherwise, it returns None, which means the default reqwest User-Agent will be used.
    #[inline]
    pub fn user_agent(&self) -> Option<&String> {
        self.user_agent.as_ref()
    }

    /// Sets the request timeout in seconds.
    ///
    /// This value determines the total time allowed for a request to complete,
    /// including DNS resolution, connection establishment, sending the request,
    /// and receiving the response.
    ///
    /// # Arguments
    ///
    /// * `timeout_seconds` - The timeout value in seconds
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn with_timeout_seconds(&mut self, timeout_seconds: u64) -> &mut Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Sets the connection timeout in seconds.
    ///
    /// This value determines the maximum time allowed to establish a connection to the server.
    /// It is a subset of the overall request timeout.
    ///
    /// # Arguments
    ///
    /// * `connect_timeout_seconds` - The connection timeout value in seconds
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn with_connect_timeout_seconds(&mut self, connect_timeout_seconds: u64) -> &mut Self {
        self.connect_timeout_seconds = connect_timeout_seconds;
        self
    }

    /// Sets the HTTP proxy URL.
    ///
    /// If set, all HTTP requests will be routed through this proxy server.
    /// Supported proxy schemes include HTTP, HTTPS, and SOCKS.
    ///
    /// # Arguments
    ///
    /// * `proxy` - The proxy URL to use
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn with_proxy(&mut self, proxy: impl Into<String>) -> &mut Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// Sets the user agent string.
    ///
    /// If set, this value will be used as the User-Agent header for all requests.
    /// If not set, the default reqwest User-Agent will be used.
    ///
    /// # Arguments
    ///
    /// * `user_agent` - The user agent string to use
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn with_user_agent(&mut self, user_agent: impl Into<String>) -> &mut Self {
        self.user_agent = Some(user_agent.into());
        self
    }

    /// Builds a reqwest::Client instance based on this configuration.
    ///
    /// This method creates a new reqwest client with the configured timeouts,
    /// proxy, and user agent settings.
    ///
    /// # Returns
    ///
    /// A reqwest::Client instance configured according to this HttpConfig
    pub fn build_reqwest_client(&self) -> reqwest::Client {
        let mut client_builder = reqwest::ClientBuilder::new()
            .timeout(std::time::Duration::from_secs(self.timeout_seconds))
            .connect_timeout(std::time::Duration::from_secs(self.connect_timeout_seconds));

        if let Some(ref proxy_url) = self.proxy {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }

        if let Some(ref user_agent) = self.user_agent {
            client_builder = client_builder.user_agent(user_agent);
        }

        client_builder
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
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
