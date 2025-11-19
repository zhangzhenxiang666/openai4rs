//! 核心客户端实现、配置以及 OpenAI API 的入口点。
//!
//! 该模块提供了主要的 [`OpenAI`] 客户端结构体，这是与 OpenAI 兼容 API 交互的主要
//! 接口。它处理 HTTP 请求配置、身份验证，并提供对各种 API 端点的访问
//! 例如聊天补全、补全和模型。
//!
//! 客户端设计具有以下特点：
//! - **线程安全**: 可以在线程间安全共享。
//! - **可配置**: 支持自定义超时、重试、代理和用户代理。
//! - **异步优先**: 使用 `tokio` 和 `reqwest` 构建非阻塞操作。
//!
//! # 示例
//!
//! ## 创建客户端
//!
//! ```rust
//! use openai4rs::OpenAI;
//!
//! // 使用 API 密钥和基础 URL 创建客户端
//! let client = OpenAI::new("your-api-key", "https://api.openai.com/v1");
//! ```
//!
//! ## 使用环境变量
//!
//! ```rust
//! use openai4rs::OpenAI;
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 从 .env 文件加载环境变量
//!     dotenv().ok();
//!
//!     // 从环境变量创建客户端
//!     // 需要设置 `OPENAI_API_KEY`
//!     let client = OpenAI::from_env()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## 访问 API 端点
//!
//! 一旦拥有客户端，您可以访问不同的 API 端点：
//!
//! - [`OpenAI::chat()`] 用于聊天补全
//! - [`OpenAI::completions()`] 用于传统的文本补全
//! - [`OpenAI::models()`] 用于列出和检索模型信息

pub mod base;
pub use base::OpenAI;
