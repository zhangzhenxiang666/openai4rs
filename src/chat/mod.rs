//! Chat completions, including streaming and tool calling.
//!
//! This module provides the primary interface for interacting with the chat
//! completions API. It supports both unary (non-streaming) and streaming
//! responses, as well as tool calling (function calling) and reasoning modes.
//!
//! # Key Components
//!
//! - [`Chat`]: The main struct for performing chat completion operations.
//! - [`chat_request`]: A convenient function for creating chat request parameters.
//! - [`ChatCompletion`]: The response type for unary chat completions.
//! - [`ChatCompletionChunk`]: The response type for streaming chat completions.
//! - [`ChatCompletionMessageParam`]: Represents a message in the chat history.
//! - [`ChatCompletionTool`]: Defines a tool (function) that the model can call.
//!
//! # Examples
//!
//! ## Unary (Non-Streaming) Chat Completion
//!
//! ```rust,no_run
//! use openai4rs::*;
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     dotenv().ok();
//!     let client = OpenAI::from_env()?;
//!     let messages = vec![user!("What is Rust?")];
//!     let request = chat_request("gpt-4", &messages);
//!     let response = client.chat().create(request).await?;
//!     println!("{:#?}", response);
//!     Ok(())
//! }
//! ```
//!
//! ## Streaming Chat Completion
//!
//! ```rust,no_run
//! use openai4rs::*;
//! use futures::StreamExt;
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     dotenv().ok();
//!     let client = OpenAI::from_env()?;
//!     let messages = vec![user!("Tell me a short story.")];
//!     let request = chat_request("gpt-4", &messages);
//!     let mut stream = client.chat().create_stream(request).await?;
//!
//!     while let Some(chunk) = stream.next().await {
//!         let chunk = chunk?;
//!         if let Some(choice) = chunk.choices.first() {
//!             if let Some(content) = &choice.delta.content {
//!                 print!("{}", content);
//!             }
//!         }
//!     }
//!     Ok(())
//! }
//! ```
//!
//! ## Tool Calling (Function Calling)
//!
//! ```rust,no_run
//! use openai4rs::*;
//! use openai4rs::chat::{ChatCompletionToolParam, tool_parameters::Parameters};
//! use dotenvy::dotenv;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     dotenv().ok();
//!     let client = OpenAI::from_env()?;
//!     
//!     // Define a tool using the type-safe Parameters builder
//!     let tool_params = Parameters::object()
//!         .property(
//!             "location",
//!             Parameters::string()
//!                 .description("The city and state, e.g. San Francisco, CA")
//!                 .build()
//!         )
//!         .require("location")
//!         .build()
//!         .unwrap();
//!
//!     let tool = ChatCompletionToolParam::function(
//!         "get_current_weather",
//!         "Get the current weather in a given location",
//!         tool_params
//!     );
//!     
//!     let messages = vec![user!("What's the weather like in Boston?")];
//!     let request = chat_request("gpt-4", &messages).tools(vec![tool]);
//!     let response = client.chat().create(request).await?;
//!     
//!     // Check if the model wants to call a tool
//!     if let Some(choice) = response.choices.first() {
//!         if let Some(tool_calls) = &choice.message.tool_calls {
//!             for tool_call in tool_calls {
//!                 println!("Tool call: {:#?}", tool_call);
//!                 // Here you would actually call the function and return the result
//!             }
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```

pub mod core;
pub mod params;
pub mod tool_parameters;
pub mod types;

pub use core::Chat;
pub use params::chat_request;
pub use tool_parameters::Parameters;
pub use types::*;
