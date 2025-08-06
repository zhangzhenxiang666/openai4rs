//!
//! # openai4rs
//!
//! 一个非官方的 Rust crate，用于与兼容 OpenAI 的 API 进行交互，提供流畅且强大的异步体验。
//!
//! 该库设计旨在易于使用、线程安全且高度可配置，适用于从简单脚本到复杂、高性能服务的广泛应用。
//!
//! ## 主要特性
//!
//! - **异步优先**: 基于 `tokio` 和 `reqwest` 构建，实现非阻塞 I/O。
//! - **聊天补全**: 完全支持聊天补全 API，包括流式传输和工具调用。
//! - **文本补全**: 支持旧版的文本补全模型。
//! - **模型管理**: 列出和检索可用模型的信息。
//! - **可配置的 HTTP 客户端**: 自定义超时、重试、代理和用户代理。
//! - **线程安全**: 可在多个线程之间安全地共享客户端。
//! - **推理模式**: 对基于推理的模型提供特别支持。
//!
//! ## 快速入门
//!
//! 首先，将 `openai4rs` 添加到您的 `Cargo.toml` 中：
//!
//! ```toml
//! [dependencies]
//! openai4rs = "0.1.3"
//! tokio = { version = "1", features = ["full"] }
//! dotenvy = "0.15"
//! ```
//!
//! 然后，使用环境变量配置客户端并进行您的第一次 API 调用。
//!
//! ```rust
//! use openai4rs::{OpenAI, chat_request, user};
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
//!     let messages = vec![user!("法国的首都是哪里？")];
//!     let request = chat_request("deepseek/deepseek-chat-v3-0324:free", &messages);
//!
//!     // 获取响应
//!     let response = client.chat().create(request).await?;
//!     println!("响应: {:#?}", response);
//!
//!     Ok(())
//! }
//! ```
//!
//! 更多示例和详细用法，请参阅每个模块的文档。
//!

/// 处理聊天补全，包括流式传输和工具调用。
pub mod chat;

/// 核心客户端实现、配置和入口点。
pub mod client;
/// 库中共享的通用类型和实用程序。
pub mod common;
/// 旧版文本补全功能。
pub mod completions;
/// 错误处理和自定义错误类型。
pub mod error;
/// 用于创建请求和消息的便捷宏。
pub mod macros;
/// 用于列出和检索模型信息的模型管理。
pub mod models;
/// 实用函数和特征。
pub mod utils;
pub use chat::*;
pub use client::{Config, OpenAI};
pub use completions::completions_request;
pub use models::models_request;
pub use serde_json;
pub use utils::Apply;
