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
//! openai4rs = "0.1.8"
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

/// API modules for different OpenAI functionality.
/// Contains chat, completions, and models modules for interacting with various API endpoints.
pub mod modules;

/// Core client implementation and entry point for the OpenAI API.
/// Provides the main OpenAI struct for interacting with OpenAI-compatible APIs.
pub mod client;

/// Configuration for the OpenAI client.
/// Handles API keys, base URLs, timeouts, and other client settings.
pub mod config;

/// Common types and utilities shared within the library.
/// Contains shared data structures and utility functions used across modules.
pub mod common;

/// Error handling and custom error types.
/// Defines the error hierarchy and error handling utilities for the library.
pub mod error;

/// Interceptor functionality for modifying requests and responses.
/// Allows middleware-style processing of API requests and responses.
pub mod interceptor;

/// HTTP service layer with retry logic, error handling, and flexible request processing.
///
/// This module provides a robust HTTP transport layer for making requests to OpenAI-compatible APIs.
/// It features configurable timeouts, proxy support, automatic retry logic with exponential backoff,
/// and support for both regular JSON responses and streaming Server-Sent Events (SSE).
///
/// The service module includes components for request execution, transport handling, and response processing.
pub mod service;

/// Utility functions and traits.
/// Contains helper functions and common traits used throughout the library.
pub mod utils;

// Re-export core types and functions
pub use client::OpenAI;
pub use config::{Config, ConfigBuilder};
pub use error::OpenAIError;
pub use interceptor::*;
pub use modules::*;
pub use serde_json;
pub use service::{HttpClient, Request, RequestBuilder, Response};
pub use utils::Apply;
// Import and re-export the new procedural macros
pub mod macros {
    pub use async_trait::async_trait;
    pub use openai4rs_macro::{assistant, content, system, tool, user};
}
pub use macros::*;
