use dotenvy::dotenv;
use openai4rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Create a base client from environment variables
    let mut client = OpenAI::from_env()?;

    // Add a module-specific interceptor to the chat module
    // This interceptor will only apply to chat API calls, not to other modules like completions or models
    client.add_chat_interceptor(ChatSpecificInterceptor::new());

    let model = "Qwen/Qwen3-235B-A22B-Instruct-2507";
    let messages = vec![
        system!("You are a helpful assistant."),
        user!("Explain the benefits of using interceptors in API clients."),
    ];

    let request = chat_request(model, &messages);

    println!("Sending request with module-specific interceptors...");

    let response = client.chat().create(request).await?;

    if let Some(content) = response.content() {
        println!("\nResponse:\n{}", content);
    } else {
        println!("\nNo content in response.");
    }

    Ok(())
}

/// Chat-Specific Interceptor - Demonstrates module-specific interceptors
/// This interceptor is only applied to chat API calls, not to other API modules.
/// It tracks the number of chat requests made and adds identifying information to requests.
#[derive(Debug)]
struct ChatSpecificInterceptor {
    call_count: std::sync::atomic::AtomicUsize,
}

impl ChatSpecificInterceptor {
    /// Creates a new instance of the chat-specific interceptor
    fn new() -> Self {
        Self {
            call_count: std::sync::atomic::AtomicUsize::new(0),
        }
    }
}

#[async_trait]
impl Interceptor for ChatSpecificInterceptor {
    /// Defines the priority of this interceptor. Medium priority means it executes
    /// in the middle for requests and in reverse middle order for responses (onion model).
    fn priority(&self) -> InterceptorPriority {
        InterceptorPriority::Medium
    }

    /// Called before the request is sent. Tracks request count and adds identifying headers.
    /// This method demonstrates how to modify requests in a module-specific interceptor.
    ///
    /// # Parameters
    /// * `request` - The request to be processed
    ///
    /// # Returns
    /// The modified request or an error if processing fails
    async fn on_request(&self, request: Request) -> Result<Request, OpenAIError> {
        let count = self
            .call_count
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
            + 1;
        println!(
            "[CHAT] ChatSpecificInterceptor: Chat request #{} to {}",
            count,
            request.url()
        );

        // Add a custom header to identify this as a chat request
        let mut request = request;
        request.headers_mut().insert(
            "X-Chat-Request-ID".to_string(),
            format!("chat-req-{}", count),
        );

        Ok(request)
    }

    /// Called after a successful response is received. Logs the response information.
    /// This method demonstrates how to process responses in a module-specific interceptor.
    ///
    /// # Parameters
    /// * `response` - The response received from the API
    ///
    /// # Returns
    /// The processed response or an error if processing fails
    async fn on_response(
        &self,
        response: reqwest::Response,
    ) -> Result<reqwest::Response, OpenAIError> {
        println!(
            "[CHAT] ChatSpecificInterceptor: Processing chat response with status {}",
            response.status()
        );
        Ok(response)
    }
}
