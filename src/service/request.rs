use crate::Config;
use crate::common::types::{JsonBody, Timeout};
use http::header::{AUTHORIZATION, AsHeaderName, IntoHeaderName};
use http::{Extensions, HeaderMap, HeaderValue};
use reqwest::{Method, RequestBuilder as ReqwestRequestBuilder};
use serde_json::Value;
use std::time::Duration;

/// HTTP请求的参数，封装了通过HTTP管道发起请求所需的所有必要信息。
///
/// 此结构体包含构建和执行HTTP请求所需的功能和配置，包括URL生成、请求构建、重试逻辑和拦截器。
///
/// # 类型参数
/// * `U` - 一个函数类型，接受Config共享引用并返回String（用于URL生成）
/// * `F` - 一个函数类型，接受Config共享引用和Request并返回Request(用于自定义的请求构建)
///
pub(crate) struct RequestSpec<U, F>
where
    U: FnOnce(&Config) -> String,
    F: FnOnce(&Config, Request) -> Request,
{
    /// 基于提供的配置生成URL的函数
    /// 接受一个Config共享引用并返回请求的完整URL字符串
    pub url_fn: U,
    /// 使用特定参数配置Request的函数
    /// 接受Config共享引用和Request并返回Request以设置请求头、请求体等
    pub builder_fn: F,
}

impl<U, F> RequestSpec<U, F>
where
    U: FnOnce(&Config) -> String,
    F: FnOnce(&Config, Request) -> Request,
{
    /// 创建一个新的RequestSpec实例
    pub fn new(url_fn: U, builder_fn: F) -> Self {
        Self { url_fn, builder_fn }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    method: Method,
    url: String,
    headers: HeaderMap<HeaderValue>,
    body: Option<JsonBody>,
    extensions: Extensions,
}

impl Request {
    pub fn new(method: Method, url: String) -> Self {
        Request {
            method,
            url,
            headers: HeaderMap::new(),
            body: None,
            extensions: Extensions::new(),
        }
    }

    #[inline]
    pub fn method(&self) -> &Method {
        &self.method
    }

    #[inline]
    pub fn method_mut(&mut self) -> &mut Method {
        &mut self.method
    }

    #[inline]
    pub fn url(&self) -> &str {
        &self.url
    }

    #[inline]
    pub fn url_mut(&mut self) -> &mut String {
        &mut self.url
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    #[inline]
    pub fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }

    #[inline]
    pub fn body(&self) -> Option<&JsonBody> {
        self.body.as_ref()
    }

    #[inline]
    pub fn body_mut(&mut self) -> Option<&mut JsonBody> {
        self.body.as_mut()
    }

    #[inline]
    pub fn extensions(&self) -> &Extensions {
        &self.extensions
    }

    #[inline]
    pub fn extensions_mut(&mut self) -> &mut Extensions {
        &mut self.extensions
    }
}

impl Request {
    /// 转换为reqwest::RequestBuilder
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

/// RequestBuilder是Request的一个包装类型, 旨在提供便捷的构建http请求的方法
pub struct RequestBuilder {
    request: Request,
}

impl RequestBuilder {
    pub fn new(request: Request) -> RequestBuilder {
        RequestBuilder { request }
    }

    /// 获取请求的共享引用
    #[inline]
    pub fn request(&self) -> &Request {
        &self.request
    }

    /// 获取请求的可变引用
    #[inline]
    pub fn request_mut(&mut self) -> &mut Request {
        &mut self.request
    }

    /// 添加请求头
    #[inline]
    pub fn header<K: IntoHeaderName>(&mut self, key: K, value: HeaderValue) -> &mut Self {
        self.request.headers.insert(key, value);
        self
    }

    /// 添加认证请求头
    pub fn bearer_auth(&mut self, token: &str) -> &mut Self {
        let val = format!("Bearer {token}");
        self.request.headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&val).unwrap_or_else(|_| {
                panic!(
                    "Unable to convert `api_key` to HeaderValue, please check if its value is valid"
                )
            }),
        );
        self
    }

    /// 添加请求体字段
    pub fn body_field<K: Into<String>, V: Into<Value>>(&mut self, key: K, value: V) -> &mut Self {
        self.request
            .body
            .get_or_insert_with(JsonBody::new)
            .insert(key.into(), value.into());
        self
    }

    /// 扩展请求体字段
    pub fn body_fields(&mut self, fields: JsonBody) -> &mut Self {
        self.request
            .body
            .get_or_insert_with(JsonBody::new)
            .extend(fields);
        self
    }

    /// 设置请求超时
    #[inline]
    pub fn timeout(&mut self, timeout: Duration) -> &mut Self {
        self.request.extensions.insert(Timeout(timeout));
        self
    }

    /// 检查请求头是否存在
    #[inline]
    pub fn has_header<K: AsHeaderName>(&self, key: K) -> bool {
        self.request.headers.contains_key(key)
    }

    /// 检查请求体字段是否存在
    #[inline]
    pub fn has_body_field(&self, key: &str) -> bool {
        match self.request.body.as_ref() {
            Some(body_fields) => body_fields.contains_key(key),
            None => false,
        }
    }

    /// 获取请求
    pub fn take(self) -> Request {
        self.request
    }
}
