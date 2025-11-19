use crate::Config;
use crate::common::types::{Body, Timeout};
use http::header::{AsHeaderName, IntoHeaderName};
use http::{Extensions, HeaderMap, HeaderValue};
use reqwest::{Method, RequestBuilder as ReqwestRequestBuilder};
use serde_json::Value;
use std::time::Duration;

/// Parameters for HTTP requests that encapsulate all necessary information
/// for making a request through the HTTP pipeline.
///
/// This structure holds the functions and configuration needed to build and execute
/// an HTTP request, including URL generation, request building, retry logic, and interceptors.
///
/// # Type Parameters
/// * `U` - A function type that takes a Config reference and returns a String (for URL generation)
/// * `F` - A function type that takes a Config reference and a mutable RequestBuilder reference
///
pub(crate) struct RequestSpec<U, F>
where
    U: FnOnce(&Config) -> String,
    F: FnOnce(&Config, &mut RequestBuilder),
{
    /// Function that generates the URL based on the provided configuration
    /// Takes a Config reference and returns the complete URL string for the request
    pub url_fn: U,
    /// Function that configures the RequestBuilder with specific parameters
    /// Takes the Config and a mutable reference to RequestBuilder to set up headers, body, etc.
    pub builder_fn: F,
}

impl<U, F> RequestSpec<U, F>
where
    U: FnOnce(&Config) -> String,
    F: FnOnce(&Config, &mut RequestBuilder),
{
    /// Creates a new HttpParams instance
    pub fn new(url_fn: U, builder_fn: F) -> Self {
        Self { url_fn, builder_fn }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Request {
    method: Method,
    url: String,
    headers: HeaderMap,
    body: Option<Body>,
    extensions: Extensions,
}

impl Request {
    /// Converts this Request to a reqwest::RequestBuilder
    ///
    /// This method creates a reqwest RequestBuilder from the current Request,
    /// applying all headers, body fields, and timeout settings.
    ///
    /// # Parameters
    /// * `client` - A reference to the reqwest client to use for building the request
    ///
    /// # Returns
    /// A new ReqwestRequestBuilder instance with all the properties from this Request
    pub fn to_reqwest(&self, client: &reqwest::Client) -> ReqwestRequestBuilder {
        let mut builder = client.request(self.method.clone(), &self.url);

        for (k, v) in &self.headers {
            builder = builder.header(k, v);
        }

        if let Some(body) = &self.body {
            builder = builder.json(body);
        }

        if let Some(timeout) = self.extensions.get::<Timeout>() {
            builder = builder.timeout(timeout.0);
        }

        builder
    }
}

/// A builder for constructing HTTP requests with various components.
pub(crate) struct RequestBuilder {
    /// The underlying Request being built
    request: Request,
}

impl RequestBuilder {
    pub fn new(method: Method, url: &str) -> RequestBuilder {
        RequestBuilder {
            request: Request {
                method,
                url: url.to_string(),
                headers: HeaderMap::new(),
                body: None,
                extensions: Extensions::new(),
            },
        }
    }

    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.request.extensions
    }

    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.request.extensions
    }

    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.request.headers
    }

    #[inline]
    pub fn header<K: IntoHeaderName>(&mut self, key: K, value: HeaderValue) -> &mut Self {
        self.request.headers.insert(key, value);
        self
    }

    pub fn bearer_auth(&mut self, token: &str) -> &mut Self {
        let val = format!("Bearer {token}");
        self.request.headers.insert(
            "Authorization",
            HeaderValue::from_str(&val).unwrap_or_else(|_| {
                panic!("不能将 `api_key` 转换为 HeaderValue, 请检查其值是否合法")
            }),
        );
        self
    }

    pub fn body_field<K: Into<String>, V: Into<Value>>(&mut self, key: K, value: V) -> &mut Self {
        self.request
            .body
            .get_or_insert_with(Body::new)
            .insert(key.into(), value.into());
        self
    }
    pub fn body_fields(&mut self, fields: Body) -> &mut Self {
        self.request
            .body
            .get_or_insert_with(Body::new)
            .extend(fields);
        self
    }

    #[inline]
    pub fn timeout(&mut self, timeout: Duration) -> &mut Self {
        self.request.extensions.insert(Timeout(timeout));
        self
    }

    #[inline]
    pub fn has_header<K: AsHeaderName>(&self, key: K) -> bool {
        self.request.headers.contains_key(key)
    }

    #[inline]
    pub fn has_body_field(&self, key: &str) -> bool {
        match self.request.body.as_ref() {
            Some(body_fields) => body_fields.contains_key(key),
            None => false,
        }
    }

    pub fn build(self) -> Request {
        self.request
    }
}
