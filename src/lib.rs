//! # OpenAI4RS: An Asynchronous Rust Client for OpenAI-Compatible APIs
//!
//! `openai4rs` is an unofficial Rust crate designed for seamless interaction with
//! OpenAI-compatible APIs, offering a robust and fluent asynchronous experience.
//!
//! This library is built with ease of use, thread safety, and high configurability in mind,
//! making it suitable for a wide range of applications from simple scripts to complex,
//! high-performance services.
//!
//! ## Key Features
//!
//! - **Async-First**: Built on `tokio` and `reqwest` for non-blocking I/O operations.
//! - **Chat Completions**: Full support for the Chat Completions API, including streaming and tool calling.
//! - **Legacy Completions**: Support for legacy text completion models.
//! - **Model Management**: List and retrieve information about available models.
//! - **Configurable HTTP Client**: Customize timeouts, retries, proxies, and user agents.
//! - **Thread Safety**: The client can be safely shared across multiple threads.
//! - **Reasoning Support**: Special support for reasoning-based models.
//!
//! ## Quick Start
//!
//! First, add `openai4rs` to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! openai4rs = "0.1.6"
//! tokio = { version = "1", features = ["full"] }
//! dotenvy = "0.15"
//! ```
//!
//! Then, configure the client using environment variables and make your first API call.
//!
//! ```rust
//! use openai4rs::*;
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load .env file
//!     dotenv().ok();
//!
//!     // Create client from environment variables
//!     let client = OpenAI::from_env()?;
//!
//!     // Create a chat request
//!     let messages = vec![user!("What is the capital of France?")];
//!     let request = chat_request("Qwen/Qwen3-235B-A22B-Instruct-2507", &messages);
//!
//!     // Get the response
//!     let response = client.chat().create(request).await?;
//!     println!("Response: {:#?}", response);
//!
//!     Ok(())
//! }
//! ```
//!
//! For more examples and detailed usage, refer to the documentation of each module.
//!

/// Handles chat completions, including streaming and tool calling.
pub mod chat;
/// Core client implementation, configuration, and entry point.
pub mod client;

/// Configuration for the OpenAI client.
pub mod config;

/// Common types and utilities shared within the library.
pub mod common;
/// Legacy text completion functionality.
pub mod completions;
/// Error handling and custom error types.
pub mod error;
/// Model management for listing and retrieving model information.
pub mod models;
/// A generic HTTP service layer with retry logic, error handling, and flexible URL generation.
///
/// This module provides a robust HTTP transport layer for making requests to OpenAI-compatible APIs.
/// It features configurable timeouts, proxy support, automatic retry logic with exponential backoff,
/// and support for both regular JSON responses and streaming Server-Sent Events (SSE).
///
/// The service module has been optimized to use generic closures for URL generation, allowing
/// for more flexible URL construction compared to simple string-based approaches.
pub mod service;
/// Utility functions and traits.
pub mod utils;

// Re-export core types and functions
pub use chat::*;
pub use client::OpenAI;
pub use completions::completions_request;
pub use config::{Config, HttpConfig};
pub use error::OpenAIError;
pub use models::models_request;
pub use serde_json;
pub use utils::Apply;

// Import and re-export the new procedural macros
pub use openai4rs_macro::{assistant, content, system, tool, user};
