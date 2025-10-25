use dotenvy::dotenv;
use openai4rs::*;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let api_key = std::env::var("OPENAI_API_KEY").unwrap();
    let base_url = std::env::var("OPENAI_BASE_URL").unwrap();
    let client = Config::builder()
        .api_key(api_key)
        .base_url(base_url)
        .global_interceptor(LoggingInterceptor::new())
        .global_interceptor(RequestModifierInterceptor)
        .build_openai()?;

    let model = "Qwen/Qwen3-235B-A22B-Instruct-2507";
    let messages = vec![
        system!("You are a helpful assistant."),
        user!("What is the weather like today?"),
    ];

    let request = chat_request(model, &messages);

    println!("Sending request with interceptors...");

    let start_time = Instant::now();
    let response = client.chat().create(request).await?;
    let duration = start_time.elapsed();

    if let Some(content) = response.content() {
        println!("\nResponse (took {:?}):\n{}", duration, content);
    } else {
        println!("\nNo content in response.");
    }

    Ok(())
}

/// Logging Interceptor - Records request and response timing information
/// This interceptor demonstrates how to track API call duration by capturing
/// the start time in on_request and calculating the elapsed time in on_response.
#[derive(Debug)]
struct LoggingInterceptor {
    start_time: std::sync::Mutex<Option<Instant>>,
}

impl LoggingInterceptor {
    /// Creates a new instance of the logging interceptor
    fn new() -> Self {
        Self {
            start_time: std::sync::Mutex::new(None),
        }
    }
}

#[async_trait::async_trait]
impl Interceptor for LoggingInterceptor {
    /// Defines the priority of this interceptor. Higher priority interceptors
    /// are executed first for requests and last for responses (onion model).
    /// This logging interceptor has high priority to capture the initial request.
    fn priority(&self) -> InterceptorPriority {
        InterceptorPriority::High // High priority, executed first for requests
    }

    /// Called before the request is sent. Records the start time for duration tracking.
    ///
    /// # Parameters
    /// * `request` - The request to be processed
    ///
    /// # Returns
    /// The modified request or an error if processing fails
    async fn on_request(&self, request: Request) -> Result<Request, OpenAIError> {
        println!(
            "[LOG] LoggingInterceptor: Request starting to {}",
            request.url()
        );
        // Record the start time for duration calculation
        *self.start_time.lock().unwrap() = Some(Instant::now());
        Ok(request)
    }

    /// Called after a successful response is received. Calculates and logs the total duration.
    ///
    /// # Parameters
    /// * `response` - The response received from the API
    ///
    /// # Returns
    /// The modified response or an error if processing fails
    async fn on_response(
        &self,
        response: reqwest::Response,
    ) -> Result<reqwest::Response, OpenAIError> {
        let duration = self
            .start_time
            .lock()
            .unwrap()
            .take()
            .map(|start| start.elapsed());
        println!(
            "[LOG] LoggingInterceptor: Response received with status {}, took {:?}",
            response.status(),
            duration
        );
        Ok(response)
    }

    /// Called when an error occurs during the request/response cycle. Logs the error.
    ///
    /// # Parameters
    /// * `error` - The error that occurred
    ///
    /// # Returns
    /// The modified error or a new error if processing fails
    async fn on_error(&self, error: OpenAIError) -> Result<OpenAIError, OpenAIError> {
        println!("[LOG] LoggingInterceptor: Error occurred: {:?}", error);
        Ok(error)
    }
}

/// Request Modifier Interceptor - Demonstrates how to modify requests before they are sent
/// This interceptor shows how to intercept and potentially modify the request object
/// before it's sent to the API endpoint. It has low priority to run after other interceptors.
#[derive(Debug)]
struct RequestModifierInterceptor;

#[async_trait]
impl Interceptor for RequestModifierInterceptor {
    /// Defines the priority of this interceptor. Low priority means it executes
    /// last for requests and first for responses (onion model).
    /// This modifier interceptor has low priority to run after other interceptors.
    fn priority(&self) -> InterceptorPriority {
        InterceptorPriority::Low // Low priority, executed last for requests
    }

    /// Called before the request is sent. Demonstrates how to modify the request.
    ///
    /// # Parameters
    /// * `request` - The request to be modified
    ///
    /// # Returns
    /// The modified request or an error if processing fails
    async fn on_request(&self, request: Request) -> Result<Request, OpenAIError> {
        println!("[MODIFY] RequestModifierInterceptor: Modifying request body");
        // Here we could modify the request body, for example by adding a prefix
        // Note: In real applications, this might require more complex logic to handle JSON request bodies
        Ok(request)
    }

    /// Called after a successful response is received. Demonstrates response processing.
    ///
    /// # Parameters
    /// * `response` - The response to be processed
    ///
    /// # Returns
    /// The processed response or an error if processing fails
    async fn on_response(
        &self,
        response: reqwest::Response,
    ) -> Result<reqwest::Response, OpenAIError> {
        println!("[MODIFY] RequestModifierInterceptor: Processing response");
        Ok(response)
    }
}
