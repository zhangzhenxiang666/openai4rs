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
    pub fn new(api_key: String, base_url: String) -> Self {
        Self { api_key, base_url }
    }

    #[inline]
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    #[inline]
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    pub fn with_base_url<T: Into<String>>(&mut self, base_url: T) -> &mut Self {
        self.base_url = base_url.into();
        self
    }

    pub fn with_api_key<T: Into<String>>(&mut self, api_key: T) -> &mut Self {
        self.api_key = api_key.into();
        self
    }
}
