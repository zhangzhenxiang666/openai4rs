//! # OpenAI4RS: 用于 OpenAI 兼容 API 的异步 Rust 客户端
//!
//! `openai4rs` 是一个非官方的 Rust crate，旨在与
//! OpenAI 兼容的 API 进行无缝交互，提供强大且流畅的异步体验。
//!
//! 此库在设计时考虑了易用性、线程安全性和高度可配置性，
//! 使其适用于从简单脚本到复杂
//! 高性能服务的各种应用场景。
//!
//! ## 主要特性
//!
//! - **异步优先**: 基于 `tokio` 和 `reqwest` 构建，支持非阻塞 I/O 操作。
//! - **聊天补全**: 完整支持聊天补全 API，包括流式传输和工具调用。
//! - **传统补全**: 支持传统文本补全模型。
//! - **文本嵌入**: 生成文本的向量表示，用于搜索、聚类和其他机器学习任务。
//! - **模型管理**: 列出和检索可用模型的信息。
//! - **可配置的 HTTP 客户端**: 自定义超时、重试、代理和用户代理。
//! - **线程安全**: 客户端可以在多个线程间安全共享。
//! - **推理支持**: 对基于推理的模型提供特殊支持。
//!
//! ## 快速开始
//!
//! 首先，在 `Cargo.toml` 中添加 `openai4rs`:
//!
//! ```toml
//! [dependencies]
//! openai4rs = "0.1.9"
//! tokio = { version = "1", features = ["full"] }
//! dotenvy = "0.15"
//! ```
//!
//! 然后，使用环境变量配置客户端并进行首次 API 调用。
//!
//! ```rust, no_run
//! use openai4rs::*;
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 加载 .env 文件
//!     dotenv().ok();
//!
//!     // 从环境变量创建客户端
//!     let client = OpenAI::from_env()?;
//!
//!     // 创建聊天请求
//!     let messages = vec![user!("法国的首都是什么？")];
//!     let request = ChatParam::new("Qwen/Qwen3-235B-A22B-Instruct-2507", &messages);
//!
//!     // 获取响应
//!     let response = client.chat().create(request).await?;
//!     println!("响应: {:#?}", response);
//!
//!     // 为文本生成嵌入
//!     let embedding_request = EmbeddingsParam::new("text-embedding-ada-002", "Hello, world!");
//!     let embedding_response = client.embeddings().create(embedding_request).await?;
//!     println!("生成了 {} 个嵌入", embedding_response.len());
//!
//!     Ok(())
//! }
//! ```
//!
//! 有关更多示例和详细用法，请参阅每个模块的文档。
//!
/// OpenAI API 的核心客户端实现和入口点。
/// 提供用于与 OpenAI 兼容 API 交互的主要 OpenAI 结构体。
pub mod client;
/// 不同 OpenAI 功能的 API 模块。
/// 包含聊天、补全、嵌入和模型模块，用于与各种 API 端点交互。
pub mod modules;

/// OpenAI 客户端的配置。
/// 处理 API 密钥、基础 URL、超时和其他客户端设置。
pub mod config;

/// 库内共享的通用类型和实用工具。
/// 包含跨模块使用的共享数据结构和实用函数。
pub mod common;

/// 错误处理和自定义错误类型。
/// 定义库的错误层次结构和错误处理实用工具。
pub mod error;

/// 具有重试逻辑、错误处理和灵活请求处理的 HTTP 服务层。
///
/// 此模块为向 OpenAI 兼容 API 发送请求提供了健壮的 HTTP 传输层。
/// 它具有可配置的超时、代理支持、具有指数退避的自动重试逻辑，
/// 以及对常规 JSON 响应和流式服务器发送事件 (SSE) 的支持。
///
/// 服务模块包含用于请求执行、传输处理和响应处理的组件。
pub mod service;

/// 实用函数和 trait。
/// 包含在整个库中使用的辅助函数和通用 trait。
pub mod utils;

// 重新导出核心类型和函数
pub use client::OpenAI;
pub use config::{Config, ConfigBuilder};
pub use error::OpenAIError;
pub use http::header;
pub use http::header::{HeaderName, HeaderValue};
pub use modules::*;
pub use serde_json;
pub use service::{Request, RequestBuilder};
// 导入并重新导出新的过程宏
pub mod macros {
    pub use openai4rs_macro::{assistant, content, system, tool, user};
}
pub use macros::*;
