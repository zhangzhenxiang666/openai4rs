//! Core client implementation, configuration, and entry point for the OpenAI API.
//!
//! This module provides the main [`OpenAI`] client struct, which is the primary
//! interface for interacting with OpenAI-compatible APIs. It handles HTTP request
//! configuration, authentication, and provides access to the various API endpoints
//! such as chat completions, completions, and models.
//!
//! The client is designed to be:
//! - **Thread-safe**: Can be safely shared across multiple threads.
//! - **Configurable**: Supports custom timeouts, retries, proxies, and user agents.
//! - **Async-first**: Built for non-blocking operations using `tokio` and `reqwest`.
//!
//! # Examples
//!
//! ## Creating a client
//!
//! ```rust
//! use openai4rs::OpenAI;
//!
//! // Create a client with an API key and base URL
//! let client = OpenAI::new("your-api-key", "https://api.openai.com/v1");
//! ```
//!
//! ## Using environment variables
//!
//! ```rust
//! use openai4rs::OpenAI;
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load environment variables from a .env file
//!     dotenv().ok();
//!
//!     // Create a client from environment variables
//!     // Requires `OPENAI_API_KEY` to be set
//!     let client = OpenAI::from_env()?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Accessing API endpoints
//!
//! Once you have a client, you can access the different API endpoints:
//!
//! - [`OpenAI::chat()`] for chat completions
//! - [`OpenAI::completions()`] for legacy text completions
//! - [`OpenAI::models()`] for listing and retrieving model information

pub mod core;
pub mod http;
pub use core::{Config, OpenAI};
