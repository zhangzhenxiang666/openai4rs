//! HTTP module for the OpenAI client.
//!
//! This module provides the HTTP transport layer implementation for making requests
//! to the OpenAI API. It includes components for configuration, request execution,
//! and response handling.

pub mod client;
pub mod executor;
pub mod request;
pub mod transport;

pub use client::HttpClient;
pub use request::{Request, RequestBuilder};
pub use reqwest::Response;
