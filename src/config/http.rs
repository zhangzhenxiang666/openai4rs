use std::collections::HashMap;

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

    /// Global headers to be included in all requests
    ///
    /// These headers will be automatically added to every HTTP request made with this configuration.
    #[builder(default = HashMap::new())]
    headers: HashMap<String, String>,

    /// Global query parameters to be appended to all request URLs
    ///
    /// These query parameters will be automatically appended to every request URL.
    #[builder(default = HashMap::new())]
    querys: HashMap<String, String>,

    /// Global body fields to be included in all requests that have a body
    ///
    /// These fields will be automatically merged into the body of every request that includes a body.
    #[builder(default = HashMap::new())]
    bodys: HashMap<String, serde_json::Value>,
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
    /// use openai4rs::HttpConfig;
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

    /// Returns a reference to the global headers map.
    ///
    /// This map contains headers that will be automatically added to all requests.
    #[inline]
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    /// Returns a reference to the global query parameters map.
    ///
    /// This map contains query parameters that will be automatically appended to all request URLs.
    #[inline]
    pub fn querys(&self) -> &HashMap<String, String> {
        &self.querys
    }

    /// Returns a reference to the global body fields map.
    ///
    /// This map contains body fields that will be automatically included in all request bodies.
    #[inline]
    pub fn bodys(&self) -> &HashMap<String, serde_json::Value> {
        &self.bodys
    }

    /// Gets a specific global body field by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the body field to retrieve
    ///
    /// # Returns
    ///
    /// An Option containing a reference to the global body field value if it exists, or None otherwise
    #[inline]
    pub fn get_body(&self, key: &str) -> Option<&serde_json::Value> {
        self.bodys.get(key)
    }

    /// Gets a specific global header by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the header to retrieve
    ///
    /// # Returns
    ///
    /// An Option containing a reference to the global header value if it exists, or None otherwise
    #[inline]
    pub fn get_header(&self, key: &str) -> Option<&String> {
        self.headers.get(key)
    }

    /// Gets a specific global query parameter by key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the query parameter to retrieve
    ///
    /// # Returns
    ///
    /// An Option containing a reference to the global query parameter value if it exists, or None otherwise
    #[inline]
    pub fn get_query(&self, key: &str) -> Option<&String> {
        self.querys.get(key)
    }

    /// Adds a global header to the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The header name
    /// * `value` - The header value
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn add_header(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Removes a global header from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The header name to remove
    ///
    /// # Returns
    ///
    /// An Option containing the removed header value if it existed, or None otherwise
    pub fn remove_header(&mut self, key: &str) -> Option<String> {
        self.headers.remove(key)
    }

    /// Adds a global query parameter to the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The query parameter name
    /// * `value` - The query parameter value
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn add_query(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.querys.insert(key.into(), value.into());
        self
    }

    /// Removes a global query parameter from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The query parameter name to remove
    ///
    /// # Returns
    ///
    /// An Option containing the removed query parameter value if it existed, or None otherwise
    pub fn remove_query(&mut self, key: &str) -> Option<String> {
        self.querys.remove(key)
    }

    /// Adds a global body field to the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The body field name
    /// * `value` - The body field value
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn add_body(
        &mut self,
        key: impl Into<String>,
        value: impl Into<serde_json::Value>,
    ) -> &mut Self {
        self.bodys.insert(key.into(), value.into());
        self
    }

    /// Removes a global body field from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The body field name to remove
    ///
    /// # Returns
    ///
    /// An Option containing the removed body field value if it existed, or None otherwise
    pub fn remove_body(&mut self, key: &str) -> Option<serde_json::Value> {
        self.bodys.remove(key)
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
            headers: HashMap::new(),
            querys: HashMap::new(),
            bodys: HashMap::new(),
        }
    }
}

impl HttpConfigBuilder {
    /// Adds a global header to the configuration.
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
        let headers_map = self.headers.get_or_insert_with(HashMap::new);
        headers_map.insert(key.into(), value.into());
        self
    }

    /// Adds a global query parameter to the configuration.
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
        let query_map = self.querys.get_or_insert_with(HashMap::new);
        query_map.insert(key.into(), value.into());
        self
    }

    /// Adds a global body field to the configuration.
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
        let body_map = self.bodys.get_or_insert_with(HashMap::new);
        body_map.insert(key.into(), value.into());
        self
    }
}
