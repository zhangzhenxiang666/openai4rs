use super::http::{HttpConfig, HttpConfigBuilder};
use super::{Credentials, CredentialsBuilder};
use crate::OpenAI;
use crate::common::types::Body;
use crate::config::CredentialsBuilderError;
use http::header::IntoHeaderName;
use http::{HeaderMap, HeaderValue};
use std::fmt;
use std::time::Duration;

#[derive(Debug)]
pub enum ConfigBuildError {
    /// 必需字段缺失错误
    RequiredFieldMissing(String),
    /// 验证错误
    ValidationError(String),
}

impl fmt::Display for ConfigBuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigBuildError::RequiredFieldMissing(field) => {
                write!(f, "Required field missing: {field}")
            }
            ConfigBuildError::ValidationError(msg) => {
                write!(f, "Validation error: {msg}")
            }
        }
    }
}

impl std::error::Error for ConfigBuildError {}

// 实现From trait以适配构建器生成的错误类型
impl From<super::http::HttpConfigBuilderError> for ConfigBuildError {
    fn from(err: super::http::HttpConfigBuilderError) -> Self {
        ConfigBuildError::RequiredFieldMissing(err.to_string())
    }
}

impl From<CredentialsBuilderError> for ConfigBuildError {
    fn from(err: CredentialsBuilderError) -> Self {
        ConfigBuildError::RequiredFieldMissing(err.to_string())
    }
}

/// 包含API通信所有设置的主配置结构
pub struct Config {
    /// 包含API密钥和URL的基础配置
    credentials: Credentials,
    /// HTTP特定配置（超时、代理等）
    http: HttpConfig,
    /// 失败请求的重试次数
    retry_count: usize,
}
impl Config {
    /// 使用指定的API密钥和基础URL创建新的Config
    ///
    /// # 参数
    ///
    /// * `api_key` - 用于身份验证的API密钥
    /// * `base_url` - API请求的基础URL
    pub fn new(api_key: impl Into<String>, base_url: impl Into<String>) -> Self {
        Self {
            credentials: Credentials::new(api_key.into(), base_url.into()),
            http: HttpConfig::default(),
            retry_count: 5,
        }
    }

    /// 创建用于流畅配置的ConfigBuilder
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder {
            retry_count: 5,
            credentials_builder: CredentialsBuilder::default(),
            http_builder: HttpConfigBuilder::default(),
        }
    }

    /// 返回API密钥
    #[inline]
    pub fn api_key(&self) -> &str {
        self.credentials.api_key()
    }

    /// 返回基础URL
    #[inline]
    pub fn base_url(&self) -> &str {
        self.credentials.base_url()
    }

    /// 返回重试次数
    #[inline]
    pub fn retry_count(&self) -> usize {
        self.retry_count
    }

    /// 返回请求超时时间
    #[inline]
    pub fn timeout(&self) -> Duration {
        self.http.timeout()
    }

    /// 返回可选的代理URL
    #[inline]
    pub fn proxy(&self) -> Option<&String> {
        self.http.proxy()
    }

    /// 返回可选的自定义用户代理字符串
    #[inline]
    pub fn user_agent(&self) -> Option<&HeaderValue> {
        self.http.user_agent()
    }

    /// 返回连接超时时间
    #[inline]
    pub fn connect_timeout(&self) -> Duration {
        self.http.connect_timeout()
    }

    /// 返回对HTTP配置的引用
    #[inline]
    pub fn http(&self) -> &HttpConfig {
        &self.http
    }

    /// 返回对基础配置的引用
    #[inline]
    pub fn credentials(&self) -> &Credentials {
        &self.credentials
    }

    /// 为此配置设置新的基础URL
    ///
    /// # 参数
    ///
    /// * `base_url` - 要使用的新基础URL
    ///
    /// # 返回
    ///
    /// 用于方法链的自引用
    pub fn with_base_url<T: Into<String>>(&mut self, base_url: T) -> &mut Self {
        self.credentials.with_base_url(base_url);
        self
    }

    /// 为此配置设置新的API密钥
    ///
    /// # 参数
    ///
    /// * `api_key` - 要使用的新API密钥
    ///
    /// # 返回
    ///
    /// 用于方法链的自引用
    pub fn with_api_key<T: Into<String>>(&mut self, api_key: T) -> &mut Self {
        self.credentials.with_api_key(api_key);
        self
    }

    /// 设置失败请求的重试次数
    ///
    /// # 参数
    ///
    /// * `retry_count` - 重试次数
    ///
    /// # 返回
    ///
    /// 用于方法链的自引用
    pub fn with_retry_count(&mut self, retry_count: usize) -> &mut Self {
        self.retry_count = retry_count;
        self
    }

    /// 设置请求超时时间
    ///
    /// # 参数
    ///
    /// * `timeout` - 超时值
    ///
    /// # 返回
    ///
    /// 用于方法链的自引用
    pub fn with_timeout(&mut self, timeout: Duration) -> &mut Self {
        self.http.with_timeout(timeout);
        self
    }

    /// 设置连接超时时间
    ///
    /// # 参数
    ///
    /// * `connect_timeout` - 连接超时值
    ///
    /// # 返回
    ///
    /// 用于方法链的自引用
    pub fn with_connect_timeout(&mut self, connect_timeout: Duration) -> &mut Self {
        self.http.with_connect_timeout(connect_timeout);
        self
    }

    /// 为请求设置HTTP代理
    ///
    /// # 参数
    ///
    /// * `proxy` - 要使用的代理URL
    ///
    /// # 返回
    ///
    /// 用于方法链的自引用
    pub fn with_proxy<T: Into<String>>(&mut self, proxy: T) -> &mut Self {
        self.http.with_proxy(proxy);
        self
    }

    /// 设置自定义用户代理字符串
    ///
    /// # 参数
    ///
    /// * `user_agent` - 要使用的用户代理字符串
    ///
    /// # 返回
    ///
    /// 用于方法链的自引用
    pub fn with_user_agent(&mut self, user_agent: HeaderValue) -> &mut Self {
        self.http.with_user_agent(user_agent);
        self
    }
}

/// 使用流畅API创建Config实例的构建器
pub struct ConfigBuilder {
    /// 失败请求的重试次数
    retry_count: usize,
    /// BaseConfig的构建器
    credentials_builder: CredentialsBuilder,
    /// HttpConfig的构建器
    http_builder: HttpConfigBuilder,
}

impl ConfigBuilder {
    /// 从当前构建器状态构建Config实例
    ///
    /// # 返回
    ///
    /// 包含Config实例或ConfigBuildError的Result
    pub fn build(self) -> Result<Config, ConfigBuildError> {
        Ok(Config {
            credentials: self.credentials_builder.build()?,
            http: self.http_builder.build()?,
            retry_count: self.retry_count,
        })
    }

    /// 从当前配置构建OpenAI客户端实例
    ///
    /// # 返回
    ///
    /// 包含OpenAI客户端实例或ConfigBuildError的Result
    pub fn build_openai(self) -> Result<OpenAI, ConfigBuildError> {
        Ok(OpenAI::with_config(self.build()?))
    }

    /// 设置配置的API密钥
    ///
    /// # 参数
    ///
    /// * `api_key` - 要使用的API密钥
    ///
    /// # 返回
    ///
    /// 用于方法链的构建器实例
    pub fn api_key<T: Into<String>>(mut self, api_key: T) -> Self {
        self.credentials_builder = self.credentials_builder.api_key(api_key.into());
        self
    }

    /// 设置配置的基础URL
    ///
    /// # 参数
    ///
    /// * `base_url` - 要使用的基础URL
    ///
    /// # 返回
    ///
    /// 用于方法链的构建器实例
    pub fn base_url<T: Into<String>>(mut self, base_url: T) -> Self {
        self.credentials_builder = self.credentials_builder.base_url(base_url.into());
        self
    }

    /// 设置配置的重试次数
    ///
    /// # 参数
    ///
    /// * `retry_count` - 重试次数
    ///
    /// # 返回
    ///
    /// 用于方法链的构建器实例
    pub fn retry_count(mut self, retry_count: usize) -> Self {
        self.retry_count = retry_count;
        self
    }

    /// 设置配置的请求超时时间
    ///
    /// # 参数
    ///
    /// * `timeout` - 超时值
    ///
    /// # 返回
    ///
    /// 用于方法链的构建器实例
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.http_builder = self.http_builder.timeout(timeout);
        self
    }

    /// 设置配置的连接超时时间
    ///
    /// # 参数
    ///
    /// * `connect_timeout` - 连接超时值
    ///
    /// # 返回
    ///
    /// 用于方法链的构建器实例
    pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
        self.http_builder = self.http_builder.connect_timeout(connect_timeout);
        self
    }

    /// 为配置设置HTTP代理
    ///
    /// # 参数
    ///
    /// * `proxy` - 要使用的代理URL
    ///
    /// # 返回
    ///
    /// 用于方法链的构建器实例
    pub fn proxy<T: Into<String>>(mut self, proxy: T) -> Self {
        self.http_builder = self.http_builder.proxy(proxy.into());
        self
    }

    /// 为配置设置自定义用户代理字符串
    ///
    /// # 参数
    ///
    /// * `user_agent` - 要使用的用户代理字符串
    ///
    /// # 返回
    ///
    /// 用于方法链的构建器实例
    pub fn user_agent(mut self, user_agent: HeaderValue) -> Self {
        self.http_builder = self.http_builder.user_agent(user_agent);
        self
    }

    /// 向HTTP配置添加全局头。
    ///
    /// # 参数
    ///
    /// * `key` - 头名称
    /// * `value` - 头值
    ///
    /// # 返回
    ///
    /// 用于方法链的构建器实例
    pub fn header<K: IntoHeaderName>(mut self, key: K, value: HeaderValue) -> Self {
        self.http_builder = self.http_builder.header(key, value);
        self
    }

    /// 向HTTP配置添加全局主体字段。
    ///
    /// # 参数
    ///
    /// * `key` - 主体字段名称
    /// * `value` - 主体字段值
    ///
    /// # 返回
    ///
    /// 用于方法链的构建器实例
    pub fn body<T: Into<String>, U: Into<serde_json::Value>>(mut self, key: T, value: U) -> Self {
        self.http_builder = self.http_builder.body(key.into(), value.into());
        self
    }

    /// 在HTTP配置中设置多个全局头。
    ///
    /// # 参数
    ///
    /// * `headers` - 头名称到值的映射
    ///
    /// # 返回
    ///
    /// 用于方法链的构建器实例
    pub fn headers(mut self, headers: HeaderMap) -> Self {
        self.http_builder = self.http_builder.headers(headers);
        self
    }

    /// 在HTTP配置中设置多个全局主体字段。
    ///
    /// # 参数
    ///
    /// * `bodys` - 主体字段名称到值的映射
    ///
    /// # 返回
    ///
    /// 用于方法链的构建器实例
    pub fn bodys(mut self, bodys: Body) -> Self {
        self.http_builder = self.http_builder.bodys(bodys);
        self
    }
}
