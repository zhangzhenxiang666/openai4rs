use reqwest::{Method, RequestBuilder as ReqwestRequestBuilder};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

#[derive(Debug, Clone)]
/// Represents an HTTP request with all its components.
pub struct Request {
    /// The HTTP method for the request (GET, POST, PUT, DELETE, etc.)
    method: Method,
    /// The URL for the request
    url: String,
    /// Headers to be included in the request
    headers: HashMap<String, String>,
    /// Query parameters to be appended to the URL
    query_params: HashMap<String, String>,
    /// Optional body fields to be included in the request body
    body_fields: Option<HashMap<String, Value>>,
    /// Optional timeout for the request
    timeout: Option<Duration>,
}

impl Request {
    #[inline]
    pub fn method(&self) -> &Method {
        &self.method
    }

    #[inline]
    pub fn url(&self) -> &str {
        &self.url
    }

    #[inline]
    pub fn headers(&self) -> &HashMap<String, String> {
        &self.headers
    }

    #[inline]
    pub fn query_params(&self) -> &HashMap<String, String> {
        &self.query_params
    }

    #[inline]
    pub fn body(&self) -> Option<&HashMap<String, Value>> {
        self.body_fields.as_ref()
    }

    #[inline]
    pub fn url_mut(&mut self) -> &mut String {
        &mut self.url
    }

    #[inline]
    pub fn headers_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.headers
    }

    #[inline]
    pub fn query_params_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.query_params
    }

    #[inline]
    pub fn body_mut(&mut self) -> &mut Option<HashMap<String, Value>> {
        &mut self.body_fields
    }

    #[inline]
    pub fn timeout(&self) -> Option<&Duration> {
        self.timeout.as_ref()
    }

    #[inline]
    pub fn timeout_mut(&mut self) -> &mut Option<Duration> {
        &mut self.timeout
    }
}

/// URL-encodes a string by percent-encoding special characters.
///
/// This function implements URL encoding (percent encoding) for query parameters
/// and other URL components. It encodes all characters except unreserved characters
/// (alphanumeric characters, hyphen, underscore, period, and tilde).
///
/// # Arguments
///
/// * `input` - The string to be URL-encoded
///
/// # Returns
///
/// A new String with special characters percent-encoded
fn url_encode(input: &str) -> String {
    let mut result = String::new();
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}

/// A builder for constructing HTTP requests with various components.
pub struct RequestBuilder {
    /// The underlying Request being built
    request: Request,
}

impl RequestBuilder {
    /// Creates a new RequestBuilder with the specified HTTP method and base URL.
    ///
    /// # Arguments
    ///
    /// * `method` - The HTTP method for the request (GET, POST, PUT, etc.)
    /// * `base_url` - The base URL for the request
    ///
    /// # Returns
    ///
    /// A new RequestBuilder instance initialized with the specified method and URL
    pub fn new(method: Method, base_url: &str) -> RequestBuilder {
        RequestBuilder {
            request: Request {
                method,
                url: base_url.to_string(),
                headers: HashMap::new(),
                query_params: HashMap::new(),
                body_fields: Some(HashMap::new()),
                timeout: None,
            },
        }
    }

    /// Adds a header to the request.
    ///
    /// # Arguments
    ///
    /// * `key` - The header name
    /// * `value` - The header value
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn header(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.request.headers_mut().insert(key.into(), value.into());
        self
    }

    /// Sets the Bearer authentication token.
    ///
    /// This adds an 'Authorization' header with the value 'Bearer {token}'.
    ///
    /// # Arguments
    ///
    /// * `token` - The authentication token
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn bearer_auth(&mut self, token: &str) -> &mut Self {
        self.request
            .headers_mut()
            .insert("Authorization".to_string(), format!("Bearer {}", token));
        self
    }

    /// Adds a query parameter to the request.
    ///
    /// # Arguments
    ///
    /// * `key` - The query parameter name
    /// * `value` - The query parameter value
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn query(&mut self, key: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.request
            .query_params_mut()
            .insert(key.into(), value.into());
        self
    }

    /// Adds a field to the request body.
    ///
    /// # Arguments
    ///
    /// * `key` - The body field name
    /// * `value` - The body field value
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn body_field(&mut self, key: impl Into<String>, value: impl Into<Value>) -> &mut Self {
        self.request
            .body_mut()
            .get_or_insert_with(HashMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Adds multiple fields to the request body.
    ///
    /// # Arguments
    ///
    /// * `fields` - A map of field names to values to add to the body
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn body_fields(&mut self, fields: HashMap<String, Value>) -> &mut Self {
        self.request
            .body_mut()
            .get_or_insert_with(HashMap::new)
            .extend(fields);
        self
    }

    /// Sets the entire request body as a map of fields.
    ///
    /// # Arguments
    ///
    /// * `body_map` - A map representing the complete request body
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn body_fields_map(&mut self, body_map: HashMap<String, Value>) -> &mut Self {
        *self.request.body_mut() = Some(body_map);
        self
    }

    /// Clears the request body.
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn without_body(&mut self) -> &mut Self {
        *self.request.body_mut() = None;
        self
    }

    /// Sets the timeout for the request.
    ///
    /// # Arguments
    ///
    /// * `timeout` - The timeout duration for the request
    ///
    /// # Returns
    ///
    /// A mutable reference to self for method chaining
    pub fn timeout(&mut self, timeout: Duration) -> &mut Self {
        *self.request.timeout_mut() = Some(timeout);
        self
    }

    /// Checks if a specific header exists in the request.
    ///
    /// # Arguments
    ///
    /// * `key` - The header name to check for
    ///
    /// # Returns
    ///
    /// true if the header exists, false otherwise
    #[inline]
    pub fn has_header(&self, key: &str) -> bool {
        self.request.headers().contains_key(key)
    }

    /// Checks if a specific query parameter exists in the request.
    ///
    /// # Arguments
    ///
    /// * `key` - The query parameter name to check for
    ///
    /// # Returns
    ///
    /// true if the query parameter exists, false otherwise
    #[inline]
    pub fn has_query(&self, key: &str) -> bool {
        self.request.query_params().contains_key(key)
    }

    /// Checks if a specific body field exists in the request.
    ///
    /// # Arguments
    ///
    /// * `key` - The body field name to check for
    ///
    /// # Returns
    ///
    /// true if the body field exists, false otherwise
    #[inline]
    pub fn has_body_field(&self, key: &str) -> bool {
        match self.request.body() {
            Some(body_fields) => body_fields.contains_key(key),
            None => false,
        }
    }

    /// Checks if any headers exist in the request.
    ///
    /// # Returns
    ///
    /// true if there are any headers, false otherwise
    #[inline]
    pub fn has_any_headers(&self) -> bool {
        !self.request.headers().is_empty()
    }

    /// Checks if any query parameters exist in the request.
    ///
    /// # Returns
    ///
    /// true if there are any query parameters, false otherwise
    #[inline]
    pub fn has_any_query_params(&self) -> bool {
        !self.request.query_params().is_empty()
    }

    /// Checks if any body fields exist in the request.
    ///
    /// # Returns
    ///
    /// true if there are any body fields, false otherwise
    #[inline]
    pub fn has_any_body_fields(&self) -> bool {
        match self.request.body() {
            Some(body_fields) => !body_fields.is_empty(),
            None => false,
        }
    }

    /// Builds the Request from the builder.
    ///
    /// This method finalizes the request by combining all components,
    /// including adding query parameters to the URL if present.
    ///
    /// # Returns
    ///
    /// The constructed Request instance
    pub fn build(mut self) -> Request {
        if !self.request.query_params().is_empty() {
            let separator = if self.request.url().contains('?') {
                "&"
            } else {
                "?"
            };
            let query_string: String = self
                .request
                .query_params()
                .iter()
                .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
                .collect::<Vec<_>>()
                .join("&");

            if !query_string.is_empty() {
                self.request.url_mut().push_str(separator);
                self.request.url_mut().push_str(&query_string);
            }
        }
        self.request
    }

    /// Builds a reqwest::RequestBuilder from this RequestBuilder.
    ///
    /// This method creates a reqwest::RequestBuilder instance that can be used
    /// to send the HTTP request.
    ///
    /// # Arguments
    ///
    /// * `client` - The reqwest client to use for building the request
    ///
    /// # Returns
    ///
    /// A reqwest::RequestBuilder instance ready to send the request
    pub fn build_reqwest_builder(self, client: &reqwest::Client) -> ReqwestRequestBuilder {
        let Request {
            method,
            url,
            headers,
            body_fields,
            timeout,
            ..
        } = self.build();
        let mut builder = client.request(method, url);

        for (k, v) in headers {
            builder = builder.header(k, v);
        }

        if let Some(body) = body_fields {
            builder = builder.json(&body);
        }

        if let Some(timeout_val) = timeout {
            builder = builder.timeout(timeout_val);
        }

        builder
    }
}
