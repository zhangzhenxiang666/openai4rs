//! Configuration module for OpenAI client
//!
//! This module provides the necessary configuration structures and builders for setting up
//! the OpenAI client with various options such as API keys, base URLs, HTTP settings,
//! timeouts, and retry policies.
//!
//! # Key Components
//!
//! - [`BaseConfig`]: Contains essential API connection parameters like API key and base URL
//! - [`HttpConfig`]: Handles HTTP-specific settings like timeouts, proxy, and user agent
//! - [`Config`]: Combines base and HTTP configurations with additional client-specific options
//! - [`ConfigBuilder`]: Provides a fluent API for constructing configurations

/// Base configuration containing essential API connection parameters
pub mod base;
/// Client configuration combining base and HTTP settings with additional options
pub mod client;
/// HTTP client configuration for connecting to API services
pub mod http;

pub use base::{BaseConfig, BaseConfigBuilder};
pub use client::{Config, ConfigBuilder};
pub use http::{HttpConfig, HttpConfigBuilder};
