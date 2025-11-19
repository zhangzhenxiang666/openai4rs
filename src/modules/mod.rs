/// Handles chat completions, including streaming and tool calling.
pub mod chat;
/// Legacy text completion functionality.
pub mod completions;
/// Text embedding functionality.
pub mod embeddings;
/// Model management for listing and retrieving model information.
pub mod models;

/// Re-exports for easier access to module functionalities.
pub use chat::Chat;
pub use chat::ChatParam;
pub use chat::tool_parameters::Parameters;
pub use chat::types::*;
pub use completions::CompletionsParam;
pub use embeddings::EmbeddingsParam;
pub use models::ModelsParam;
