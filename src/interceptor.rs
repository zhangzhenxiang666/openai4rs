//! Interceptor module for OpenAI client
//!
//! This module provides the necessary structures and traits for implementing
//! request/response interceptors that can modify requests before they are sent
//! and responses after they are received.

use crate::error::OpenAIError;
use crate::service::request::Request;
use reqwest::Response;
use std::fmt::Debug;
use std::sync::Arc;

/// Interceptor trait that defines methods for intercepting requests and responses
///
/// # Example
///
/// ```rust
/// use openai4rs::interceptor::{Interceptor, InterceptorPriority};
/// use openai4rs::error::OpenAIError;
/// use openai4rs::service::{Request, Response};
/// use openai4rs::async_trait;
///
/// #[derive(Debug)]
/// struct LoggingInterceptor;
///
/// #[async_trait]
/// impl Interceptor for LoggingInterceptor {
///     fn priority(&self) -> InterceptorPriority {
///         InterceptorPriority::Medium
///     }
///
///     async fn on_request(&self, request: Request) -> Result<Request, OpenAIError> {
///         println!("Sending request to: {}", request.url());
///         Ok(request)
///     }
///
///     async fn on_response(&self, response: Response) -> Result<Response, OpenAIError> {
///         println!("Received response with status: {}", response.status());
///         Ok(response)
///     }
///
///     async fn on_error(&self, error: OpenAIError) -> Result<OpenAIError, OpenAIError> {
///         eprintln!("Interceptor caught error: {:?}", error);
///         Ok(error)
///     }
/// }
///
/// #[derive(Debug)]
/// struct AuthInterceptor {
///     api_key: String,
/// }
///
/// #[async_trait]
/// impl Interceptor for AuthInterceptor {
///     async fn on_request(&self, request: Request) -> Result<Request, OpenAIError> {
///         // Add authentication header to request
///         let mut request = request;
///         request.headers_mut().insert(
///             "Authorization".to_string(),
///             format!("Bearer {}", &self.api_key).parse().unwrap()
///         );
///         Ok(request)
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait Interceptor: Send + Sync {
    /// Get the priority of this interceptor. This determines the order in which
    /// interceptors are executed.
    ///
    /// # Returns
    /// The priority of this interceptor
    fn priority(&self) -> InterceptorPriority {
        InterceptorPriority::Medium
    }

    /// Called before the request is sent. Allows modification of the request.
    ///
    /// # Parameters
    /// * `request` - The request to be modified (takes ownership)
    ///
    /// # Returns
    /// A Result containing the modified request or an error. If an error is returned,
    /// the interceptor chain will be interrupted.
    async fn on_request(&self, request: Request) -> Result<Request, OpenAIError> {
        Ok(request)
    }

    /// Called after a successful response is received. Allows modification of the response.
    ///
    /// # Parameters
    /// * `response` - The response to be modified (takes ownership)
    ///
    /// # Returns
    /// A Result containing the modified response or an error. If an error is returned,
    /// the interceptor chain will be interrupted.
    async fn on_response(&self, response: Response) -> Result<Response, OpenAIError> {
        Ok(response)
    }

    /// Called when an error occurs during the request/response cycle.
    ///
    /// # Parameters
    /// * `error` - The error that occurred (takes ownership)
    ///
    /// # Returns
    /// A Result containing the modified error or a new error. If an error is returned,
    /// the interceptor chain will be interrupted.
    async fn on_error(&self, error: OpenAIError) -> Result<OpenAIError, OpenAIError> {
        Ok(error)
    }
}

/// Priority levels for interceptors to control execution order
#[derive(Debug, Clone)]
pub enum InterceptorPriority {
    /// Highest priority - executed first for requests, last for responses
    Highest,
    /// High priority
    High,
    /// Medium priority (default)
    Medium,
    /// Low priority
    Low,
    /// Lowest priority - executed last for requests, first for responses
    Lowest,
    /// Custom priority - allows fine-grained control
    Custom(i32),
}

impl InterceptorPriority {
    /// Convert to integer value for ordering
    fn to_int(&self) -> i32 {
        match self {
            InterceptorPriority::Highest => 100,
            InterceptorPriority::High => 75,
            InterceptorPriority::Medium => 50,
            InterceptorPriority::Low => 25,
            InterceptorPriority::Lowest => 0,
            InterceptorPriority::Custom(value) => *value,
        }
    }
}

impl PartialEq for InterceptorPriority {
    fn eq(&self, other: &Self) -> bool {
        self.to_int() == other.to_int()
    }
}

impl Eq for InterceptorPriority {}

impl PartialOrd for InterceptorPriority {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for InterceptorPriority {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_int().cmp(&other.to_int())
    }
}

/// A prioritized interceptor that combines an interceptor with its priority
pub struct PrioritizedInterceptor {
    /// The interceptor instance
    interceptor: Arc<dyn Interceptor>,
    /// The priority of this interceptor
    priority: InterceptorPriority,
}

impl PrioritizedInterceptor {
    /// Create a new prioritized interceptor by wrapping an interceptor with its priority
    ///
    /// # Parameters
    /// * `interceptor` - The interceptor to wrap, wrapped in an Arc for thread safety
    ///
    /// # Returns
    /// A new PrioritizedInterceptor instance with the interceptor and its priority
    pub fn new(interceptor: Arc<dyn Interceptor>) -> Self {
        let priority = interceptor.priority();
        Self {
            interceptor,
            priority,
        }
    }

    /// Get a reference to the wrapped interceptor
    ///
    /// # Returns
    /// A reference to the Arc<dyn Interceptor> that contains the actual interceptor
    #[inline]
    pub fn interceptor(&self) -> &Arc<dyn Interceptor> {
        &self.interceptor
    }

    /// Get a reference to the priority of this interceptor
    ///
    /// # Returns
    /// A reference to the InterceptorPriority that determines execution order
    #[inline]
    pub fn priority(&self) -> &InterceptorPriority {
        &self.priority
    }

    /// Get a mutable reference to the wrapped interceptor
    ///
    /// # Returns
    /// A mutable reference to the Arc<dyn Interceptor>
    #[inline]
    pub fn interceptor_mut(&mut self) -> &mut Arc<dyn Interceptor> {
        &mut self.interceptor
    }

    /// Get a mutable reference to the priority of this interceptor
    ///
    /// # Returns
    /// A mutable reference to the InterceptorPriority
    #[inline]
    pub fn priority_mut(&mut self) -> &mut InterceptorPriority {
        &mut self.priority
    }
}

/// A chain of interceptors that can be executed in a specific order
#[derive(Default)]
pub struct InterceptorChain(Vec<PrioritizedInterceptor>);

impl InterceptorChain {
    /// Create a new empty interceptor chain
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Add an interceptor to the chain
    pub fn add_interceptor(&mut self, interceptor: impl Interceptor + 'static) {
        self.0
            .push(PrioritizedInterceptor::new(Arc::new(interceptor)));
        // Sort interceptors by priority (highest first for requests)
        self.0.sort_by(|a, b| b.priority().cmp(a.priority()));
    }

    /// Get the number of interceptors in the chain
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Execute request interceptors in priority order (highest to lowest)
    /// Returns an error if any interceptor fails
    pub async fn execute_request_interceptors(
        &self,
        request: Request,
    ) -> Result<Request, OpenAIError> {
        let mut current_request = request;
        for prioritized_interceptor in &self.0 {
            current_request = prioritized_interceptor
                .interceptor()
                .on_request(current_request)
                .await?;
        }
        Ok(current_request)
    }

    /// Execute response interceptors in reverse priority order (lowest to highest)
    /// This follows the onion model where the response flows back in reverse order
    /// Returns an error if any interceptor fails
    pub async fn execute_response_interceptors(
        &self,
        response: Response,
    ) -> Result<Response, OpenAIError> {
        let mut current_response = response;
        // Process in reverse order (lowest priority first)
        for prioritized_interceptor in self.0.iter().rev() {
            current_response = prioritized_interceptor
                .interceptor()
                .on_response(current_response)
                .await?;
        }
        Ok(current_response)
    }

    /// Execute error interceptors in reverse priority order (lowest to highest)
    /// This follows the onion model where the error flows back in reverse order
    /// Returns an error if any interceptor fails
    pub async fn execute_error_interceptors(
        &self,
        error: OpenAIError,
    ) -> Result<OpenAIError, OpenAIError> {
        let mut current_error = error;
        // Process in reverse order (lowest priority first)
        for prioritized_interceptor in self.0.iter().rev() {
            current_error = prioritized_interceptor
                .interceptor()
                .on_error(current_error)
                .await?;
        }
        Ok(current_error)
    }
}

impl std::ops::Index<usize> for InterceptorChain {
    type Output = PrioritizedInterceptor;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl std::ops::IndexMut<usize> for InterceptorChain {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug)]
    struct TestInterceptor {
        id: String,
        priority: InterceptorPriority,
    }

    #[async_trait::async_trait]
    impl Interceptor for TestInterceptor {
        fn priority(&self) -> InterceptorPriority {
            self.priority.clone()
        }

        async fn on_request(&self, request: Request) -> Result<Request, OpenAIError> {
            println!("TestInterceptor {} processing request", self.id);
            Ok(request)
        }

        async fn on_response(&self, response: Response) -> Result<Response, OpenAIError> {
            println!("TestInterceptor {} processing response", self.id);
            Ok(response)
        }

        async fn on_error(&self, error: OpenAIError) -> Result<OpenAIError, OpenAIError> {
            println!("TestInterceptor {} processing error", self.id);
            Ok(error)
        }
    }

    #[tokio::test]
    async fn test_interceptor_chain_order() {
        let mut chain = InterceptorChain::new();

        let low_interceptor = TestInterceptor {
            id: "low".to_string(),
            priority: InterceptorPriority::Low,
        };
        let high_interceptor = TestInterceptor {
            id: "high".to_string(),
            priority: InterceptorPriority::High,
        };
        let medium_interceptor = TestInterceptor {
            id: "medium".to_string(),
            priority: InterceptorPriority::Medium,
        };

        chain.add_interceptor(low_interceptor);
        chain.add_interceptor(high_interceptor);
        chain.add_interceptor(medium_interceptor);

        // Should have 3 interceptors
        assert_eq!(chain.len(), 3);

        // The first interceptor in the list should be the highest priority
        assert_eq!(chain[0].priority, InterceptorPriority::High);
        assert_eq!(chain[1].priority, InterceptorPriority::Medium);
        assert_eq!(chain[2].priority, InterceptorPriority::Low);
    }
}
