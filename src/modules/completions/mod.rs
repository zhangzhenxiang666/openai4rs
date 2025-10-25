//! Legacy text completions.
//!
//! This module provides support for the legacy text completions API. While
//! chat completions are generally preferred for new projects, this module
//! is available for compatibility with older models or specific use cases
//! that require the text completions format.
//!
//! # Key Components
//!
//! - [`Completions`]: The main struct for performing text completion operations.
//! - [`completions_request`]: A convenient function for creating completion request parameters.
//! - [`Completion`]: The response type for text completions.
//!
//! # Examples
//!
//! ## Unary (Non-Streaming) Text Completion
//!
//! ```rust,no_run
//! use openai4rs::{OpenAI, completions_request};
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     dotenv().ok();
//!     let client = OpenAI::from_env()?;
//!     let response = client
//!         .completions()
//!         .create(completions_request("text-davinci-003", "Write a poem about Rust.").max_tokens(100))
//!         .await?;
//!
//!     println!("{:#?}", response);
//!     Ok(())
//! }
//! ```
//!
//! ## Streaming Text Completion
//!
//! ```rust,no_run
//! use openai4rs::{OpenAI, completions_request};
//! use futures::StreamExt;
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     dotenv().ok();
//!     let client = OpenAI::from_env()?;
//!     let mut stream = client
//!         .completions()
//!         .create_stream(completions_request("text-davinci-003", "Write a story about a robot.").max_tokens(100))
//!         .await?;
//!
//!     while let Some(chunk) = stream.next().await {
//!         let chunk = chunk?;
//!         if let Some(choice) = chunk.choices.first() {
//!             print!("{}", choice.text);
//!         }
//!     }
//!     Ok(())
//! }
//! ```

pub mod handler;
pub mod params;
pub mod types;

pub use handler::Completions;
pub use params::completions_request;
pub use types::Completion;
