//! Model management for listing and retrieving model information.
//!
//! This module provides functionality for interacting with the models API,
//! allowing you to list available models or retrieve detailed information
//! about a specific model.
//!
//! # Key Components
//!
//! - [`Models`]: The main struct for performing model operations.
//! - [`models_request`]: A convenient function for creating model request parameters.
//! - [`ModelsData`]: The response type for listing models.
//! - [`Model`]: The response type for retrieving a specific model's information.
//!
//! # Examples
//!
//! ## Listing Models
//!
//! ```rust,no_run
//! use openai4rs::{OpenAI, models_request};
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     dotenv().ok();
//!     let client = OpenAI::from_env()?;
//!     let models = client.models().list(models_request()).await?;
//!
//!     for model in models.data {
//!         println!("Model ID: {}", model.id);
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Retrieving a Specific Model
//!
//! ```rust,no_run
//! use openai4rs::{OpenAI, models_request};
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     dotenv().ok();
//!     let client = OpenAI::from_env()?;
//!     let model = client.models().retrieve("gpt-4", models_request()).await?;
//!
//!     println!("Model details: {:#?}", model);
//!     Ok(())
//! }
//! ```

pub mod handler;
pub mod params;
pub mod types;

pub use handler::Models;
pub use params::models_request;
pub use types::*;
