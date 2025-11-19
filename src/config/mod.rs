//! OpenAI客户端的配置模块
//!
//! 此模块提供了设置OpenAI客户端所需的各种配置结构和构建器，包括API密钥、基础URL、HTTP设置、
//! 超时时间和重试策略等选项。
//!
//! # 主要组件
//!
//! - [`Credentials`]: 包含API密钥和基础URL等基本API连接参数
//! - [`HttpConfig`]: 处理HTTP特定设置，如超时、代理和用户代理
//! - [`Config`]: 结合基础和HTTP配置以及额外的客户端特定选项
//! - [`ConfigBuilder`]: 提供流畅的API来构建配置
//!
/// 客户端配置，结合基础和HTTP设置以及额外选项
pub mod client;
/// 用于连接API服务的HTTP客户端配置
pub mod http;

pub use client::{Config, ConfigBuilder};
use derive_builder::Builder;
pub use http::{HttpConfig, HttpConfigBuilder};

#[derive(Debug, Clone, Builder)]
#[builder(name = "CredentialsBuilder", pattern = "owned", setter(strip_option))]
pub struct Credentials {
    /// 用于服务身份验证的API密钥
    api_key: String,
    /// API请求的基础URL（例如，"https://api.openai.com/v1"）
    base_url: String,
}

impl Credentials {
    /// 使用指定的API密钥和基础URL创建新的BaseConfig
    ///
    /// # 参数
    ///
    /// * `api_key` - 用于身份验证的API密钥
    /// * `base_url` - API请求的基础URL
    pub fn new(api_key: String, base_url: String) -> Self {
        Self { api_key, base_url }
    }

    /// 返回基础URL的引用
    #[inline]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// 返回API密钥的引用
    #[inline]
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// 为当前配置设置新的基础URL
    ///
    /// # 参数
    ///
    /// * `base_url` - 要使用的新基础URL
    ///
    /// # 返回
    ///
    /// 用于方法链的可变引用
    pub fn with_base_url(&mut self, base_url: impl Into<String>) -> &mut Self {
        self.base_url = base_url.into();
        self
    }

    /// 为当前配置设置新的API密钥
    ///
    /// # 参数
    ///
    /// * `api_key` - 要使用的新API密钥
    ///
    /// # 返回
    ///
    /// 用于方法链的可变引用
    pub fn with_api_key(&mut self, api_key: impl Into<String>) -> &mut Self {
        self.api_key = api_key.into();
        self
    }
}
