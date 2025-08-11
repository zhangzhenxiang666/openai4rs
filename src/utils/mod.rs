//! Utility functions and traits used internally by the `openai4rs` crate.
//!
//! This module contains various utility functions and traits that are used
//! throughout the `openai4rs` crate to simplify common tasks and provide
//! shared functionality.
//!
//! # Key Components
//!
//! - [`Apply`]: A trait for applying asynchronous functions to streams.
//! - [`ResponseHandler`]: A trait for processing API responses (used internally).
//! - [`AsyncFrom`]: A trait for asynchronous conversion between types (used internally).

pub use traits::Apply;

pub mod methods;
pub mod traits;
