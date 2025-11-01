//! Embeddings for generating vector representations of text.
//!
//! This module provides the primary interface for interacting with the embeddings API.
//! It generates vector representations of text inputs that can be used for search,
//! clustering, and other machine learning tasks.
//!
//! # Key Components
//!
//! - [`Embeddings`]: The main struct for performing embedding operations.
//! - [`embeddings_request`]: A convenient function for creating embedding request parameters.
//! - [`EmbeddingResponse`]: The response type containing the generated embeddings.
//! - [`Input`]: Represents the input text to embed, either as a single string or multiple strings.
//!
//! # Examples
//!
//! ## Basic Embedding Generation
//!
//! ```rust,no_run
//! use openai4rs::*;
//! use openai4rs::embeddings::params::embeddings_request;
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     dotenv().ok();
//!     let client = OpenAI::from_env()?;
//!     let request = embeddings_request("text-embedding-ada-002", "Hello, world!");
//!     let response = client.embeddings().create(request).await?;
//!     println!("{:#?}", response);
//!     Ok(())
//! }
//! ```
//!
//! ## Multiple Text Embeddings
//!
//! ```rust,no_run
//! use openai4rs::*;
//! use openai4rs::embeddings::params::embeddings_request;
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     dotenv().ok();
//!     let client = OpenAI::from_env()?;
//!     let texts = vec!["Hello, world!", "How are you?", "Rust is awesome!"];
//!     let request = embeddings_request("text-embedding-ada-002", texts);
//!     let response = client.embeddings().create(request).await?;
//!     println!("Generated {} embeddings", response.len());
//!     for (i, embedding) in response.embeddings().iter().enumerate() {
//!         println!("Embedding {}: {} dimensions", i, embedding.dimensions());
//!     }
//!     Ok(())
//! }
//! ```

pub mod handler;
pub mod params;
pub mod types;

pub use handler::Embeddings;
pub use params::embeddings_request;
pub use types::{EmbeddingResponse, Input};
