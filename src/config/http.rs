use crate::common::types::Body;
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
    #[builder(default = Body::new())]
    bodys: Body,
}

impl HttpConfig {
    /// 创建一个新的配置构建器。
    ///
    /// 这是构建HttpConfig的首选方式，允许
    /// 使用合理默认值进行灵活配置。
    ///
    /// # 示例
    ///
    /// ```
    /// use openai4rs::config::HttpConfig;
    /// use std::time::Duration;
    /// let config = HttpConfig::builder()
    ///     .timeout(Duration::from_secs(300))
    ///     .proxy("http://proxy.example.com:8080".to_string())
    ///     .build()
    ///     .unwrap();
    /// ```
    pub fn builder() -> HttpConfigBuilder {
        HttpConfigBuilder::default()
    }

    /// 返回请求超时时间。
    ///
    /// 此值确定请求完成的总允许时间，
    /// 包括DNS解析、连接建立、发送请求
    /// 和接收响应。
    #[inline]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// 返回连接超时时间。
    ///
    /// 此值确定建立与服务器连接的最大允许时间。
    /// 它是整体请求超时的一个子集。
    #[inline]
    pub fn connect_timeout(&self) -> Duration {
        self.connect_timeout
    }

    /// 返回对代理URL的可选引用。
    ///
    /// 如果配置了代理，此方法返回包含对代理URL引用的Some。
    /// 否则，返回None。
    #[inline]
    pub fn proxy(&self) -> Option<&String> {
        self.proxy.as_ref()
    }

    /// 返回对用户代理字符串的可选引用。
    ///
    /// 如果配置了自定义用户代理，此方法返回包含对用户代理字符串引用的Some。
    /// 否则，返回None，这意味着将使用默认的reqwest用户代理。
    #[inline]
    pub fn user_agent(&self) -> Option<&HeaderValue> {
        self.headers.get(USER_AGENT)
    }

    /// 返回对全局头映射的引用。
    ///
    /// 此映射包含将自动添加到所有请求的头。
    #[inline]
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// 返回对全局请求体字段映射的引用。
    ///
    /// 此映射包含将自动包含在所有请求体中的请求体字段。
    #[inline]
    pub fn bodys(&self) -> &Body {
        &self.bodys
    }

    /// 按键获取特定的全局请求体字段。
    ///
    /// # 参数
    ///
    /// * `key` - 要检索的请求体字段的键
    ///
    /// # 返回值
    ///
    /// 如果存在，则包含对全局请求体字段值的引用的Option，否则为None
    #[inline]
    pub fn get_body(&self, key: &str) -> Option<&serde_json::Value> {
        self.bodys.get(key)
    }

    /// 按键获取特定的全局头。
    ///
    /// # 参数
    ///
    /// * `key` - 要检索的头的键
    ///
    /// # 返回值
    ///
    /// 如果存在，则包含对全局头值的引用的Option，否则为None
    #[inline]
    pub fn get_header(&self, key: &str) -> Option<&HeaderValue> {
        self.headers.get(key)
    }

    /// 向配置中添加全局头。
    ///
    /// # 参数
    ///
    /// * `key` - 头名称
    /// * `value` - 头值
    ///
    /// # 返回值
    ///
    /// 用于方法链的自引用
    pub fn add_header<K: IntoHeaderName>(&mut self, key: K, value: HeaderValue) -> &mut Self {
        self.headers.insert(key, value);
        self
    }

    /// 从配置中移除全局头。
    ///
    /// # 参数
    ///
    /// * `key` - 要移除的头名称
    ///
    /// # 返回值
    ///
    /// 如果存在，则包含移除的头值的Option，否则为None
    pub fn remove_header(&mut self, key: &str) -> Option<HeaderValue> {
        self.headers.remove(key)
    }

    /// 向配置中添加全局请求体字段。
    ///
    /// # 参数
    ///
    /// * `key` - 请求体字段名称
    /// * `value` - 请求体字段值
    ///
    /// # 返回值
    ///
    /// 用于方法链的自引用
    pub fn add_body<T: Into<String>, U: Into<serde_json::Value>>(
        &mut self,
        key: T,
        value: U,
    ) -> &mut Self {
        self.bodys.insert(key.into(), value.into());
        self
    }

    /// 从配置中移除全局请求体字段。
    ///
    /// # 参数
    ///
    /// * `key` - 要移除的请求体字段名称
    ///
    /// # 返回值
    ///
    /// 如果存在，则包含移除的请求体字段值的Option，否则为None
    pub fn remove_body(&mut self, key: &str) -> Option<serde_json::Value> {
        self.bodys.remove(key)
    }

    /// 设置请求超时时间。
    ///
    /// 此值确定请求完成的总允许时间，
    /// 包括DNS解析、连接建立、发送请求
    /// 和接收响应。
    ///
    /// # 参数
    ///
    /// * `timeout` - 超时值
    ///
    /// # 返回值
    ///
    /// 用于方法链的自引用
    pub fn with_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.timeout = timeout;
        self
    }

    /// 设置连接超时时间。
    ///
    /// 此值确定建立与服务器连接的最大允许时间。
    /// 它是整体请求超时的一个子集。
    ///
    /// # 参数
    ///
    /// * `connect_timeout` - 连接超时值
    ///
    /// # 返回值
    ///
    /// 用于方法链的自引用
    pub fn with_connect_timeout(&mut self, connect_timeout: Duration) -> &mut Self {
        self.connect_timeout = connect_timeout;
        self
    }

    /// 设置HTTP代理URL。
    ///
    /// 如果设置，所有HTTP请求将通过此代理服务器路由。
    /// 支持的代理方案包括HTTP、HTTPS和SOCKS。
    ///
    /// # 参数
    ///
    /// * `proxy` - 要使用的代理URL
    ///
    /// # 返回值
    ///
    /// 用于方法链的自引用
    pub fn with_proxy<T: Into<String>>(&mut self, proxy: T) -> &mut Self {
        self.proxy = Some(proxy.into());
        self
    }

    /// 设置用户代理字符串。
    ///
    /// 如果设置，此值将用作所有请求的User-Agent头。
    /// 如果未设置，将使用默认的reqwest用户代理。
    ///
    /// # 参数
    ///
    /// * `user_agent` - 要使用的用户代理字符串
    ///
    /// # 返回值
    ///
    /// 用于方法链的自引用
    pub fn with_user_agent(&mut self, user_agent: HeaderValue) -> &mut Self {
        self.headers.insert(USER_AGENT, user_agent);
        self
    }

    /// 根据此配置构建reqwest::Client实例。
    ///
    /// 此方法使用配置的超时、
    /// 代理和用户代理设置创建一个新的reqwest客户端。
    ///
    /// # 返回值
    ///
    /// 根据此HttpConfig配置的reqwest::Client实例
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
    /// 返回默认HTTP配置。
    ///
    /// 默认配置包括：
    /// - 300秒请求超时
    /// - 10秒连接超时
    /// - 无代理
    /// - 无自定义用户代理
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(300),
            connect_timeout: Duration::from_secs(10),
            proxy: None,
            bodys: Body::new(),
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
        let body_map = self.bodys.get_or_insert_with(Body::new);
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
