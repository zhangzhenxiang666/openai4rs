use crate::common::types::JsonBody;
use derive_builder::Builder;
use http::{
    HeaderMap, HeaderValue,
    header::{IntoHeaderName, USER_AGENT},
};
use std::time::Duration;

/// 连接到API服务的HTTP客户端配置。
///
/// 该结构体保存与底层HTTP传输层相关的设置，
/// 如超时、代理设置和用户代理。它被设计为
/// 可重用且独立于任何特定API的业务逻辑。
///
/// 该配置使用构建器模式进行灵活构建，允许
/// 用户仅设置他们需要的选项，同时对其他选项使用合理的默认值。
#[derive(Debug, Clone, Builder)]
#[builder(name = "HttpConfigBuilder", pattern = "owned", setter(strip_option))]
pub struct HttpConfig {
    /// 请求超时时间。默认值：300秒
    ///
    /// 这是请求完成的总允许时间，包括
    /// DNS解析、连接建立、发送请求
    /// 和接收响应。
    #[builder(default = Duration::from_secs(300))]
    timeout: Duration,

    /// 连接超时时间。默认值：10秒
    ///
    /// 这是建立与服务器连接的最大允许时间。
    /// 它是整体请求超时的一个子集。
    #[builder(default = Duration::from_secs(10))]
    connect_timeout: Duration,

    /// HTTP代理URL（如果有的话）
    ///
    /// 如果设置，所有HTTP请求将通过此代理服务器路由。
    /// 支持的代理方案包括HTTP、HTTPS和SOCKS。
    #[builder(default = None)]
    proxy: Option<String>,

    /// 要包含在所有请求中的全局头
    ///
    /// 这些头将自动添加到使用此配置发出的每个HTTP请求中。
    #[builder(default = HeaderMap::new())]
    headers: HeaderMap,

    /// 要包含在所有有请求体的请求中的全局请求体字段
    ///
    /// 这些字段将自动合并到每个包含请求体的请求的请求体中。
    #[builder(default = JsonBody::new())]
    bodys: JsonBody,
}

impl HttpConfig {
    pub fn builder() -> HttpConfigBuilder {
        HttpConfigBuilder::default()
    }

    #[inline]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    #[inline]
    pub fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }

    #[inline]
    pub fn proxy(&self) -> Option<&String> {
        self.proxy.as_ref()
    }

    #[inline]
    pub fn user_agent(&self) -> Option<&HeaderValue> {
        self.headers.get(USER_AGENT)
    }

    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    #[inline]
    pub fn bodys(&self) -> &JsonBody {
        &self.bodys
    }

    #[inline]
    pub fn get_body(&self, key: &str) -> Option<&serde_json::Value> {
        self.bodys.get(key)
    }

    #[inline]
    pub fn get_header(&self, key: &str) -> Option<&HeaderValue> {
        self.headers.get(key)
    }

    pub fn add_header<K: IntoHeaderName>(&mut self, key: K, value: HeaderValue) -> &mut Self {
        self.headers.insert(key, value);
        self
    }

    pub fn remove_header(&mut self, key: &str) -> Option<HeaderValue> {
        self.headers.remove(key)
    }

    pub fn add_body<T: Into<String>, U: Into<serde_json::Value>>(
        &mut self,
        key: T,
        value: U,
    ) -> &mut Self {
        self.bodys.insert(key.into(), value.into());
        self
    }

    pub fn remove_body(&mut self, key: &str) -> Option<serde_json::Value> {
        self.bodys.remove(key)
    }

    pub fn with_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }

    pub fn with_connect_timeout(&mut self, connect_timeout: Duration) -> &mut Self {
        self.connect_timeout = connect_timeout;
        self
    }

    pub fn with_proxy<T: Into<String>>(&mut self, proxy: T) -> &mut Self {
        self.proxy = Some(proxy.into());
        self
    }

    pub fn with_user_agent(&mut self, user_agent: HeaderValue) -> &mut Self {
        self.headers.insert(USER_AGENT, user_agent);
        self
    }

    pub fn build_reqwest_client(&self) -> reqwest::Client {
        let mut client_builder = reqwest::ClientBuilder::new()
            .timeout(self.timeout)
            .connect_timeout(self.connect_timeout);

        if let Some(ref proxy_url) = self.proxy {
            if let Ok(proxy) = reqwest::Proxy::all(proxy_url) {
                client_builder = client_builder.proxy(proxy);
            }
        }

        if let Some(user_agent) = self.headers.get(USER_AGENT) {
            client_builder = client_builder.user_agent(user_agent);
        }

        client_builder
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    }
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300),
            connect_timeout: Duration::from_secs(10),
            proxy: None,
            bodys: JsonBody::new(),
            headers: HeaderMap::new(),
        }
    }
}

impl HttpConfigBuilder {
    pub fn header<K: IntoHeaderName>(mut self, key: K, value: HeaderValue) -> Self {
        let headers_map = self.headers.get_or_insert_with(HeaderMap::new);
        headers_map.insert(key, value);
        self
    }

    pub fn body<T: Into<String>, U: Into<serde_json::Value>>(mut self, key: T, value: U) -> Self {
        let body_map = self.bodys.get_or_insert_with(JsonBody::new);
        body_map.insert(key.into(), value.into());
        self
    }

    pub fn user_agent(mut self, user_agent: HeaderValue) -> Self {
        self.headers
            .get_or_insert_with(HeaderMap::new)
            .insert(USER_AGENT, user_agent);
        self
    }
}
