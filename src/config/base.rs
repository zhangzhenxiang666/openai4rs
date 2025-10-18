use derive_builder::Builder;

/// Base configuration containing essential API connection parameters
#[derive(Debug, Clone, Builder)]
#[builder(name = "BaseConfigBuilder", pattern = "owned", setter(strip_option))]
pub struct BaseConfig {
    /// API key for authentication with the service
    api_key: String,
    /// Base URL for API requests (e.g., "https://api.openai.com/v1")
    base_url: String,
}

impl BaseConfig {
    /// Creates a new BaseConfig with the specified API key and base URL
    ///
    /// # Arguments
    ///
    /// * `api_key` - The API key for authentication
    /// * `base_url` - The base URL for API requests
    pub fn new(api_key: String, base_url: String) -> Self {
        Self { api_key, base_url }
    }

    /// Returns a reference to the base URL
    #[inline]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Returns a reference to the API key
    #[inline]
    pub fn api_key(&self) -> &str {
        &self.api_key
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
        self.base_url = base_url.into();
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
        self.api_key = api_key.into();
        self
    }
}
