use dotenvy::dotenv;
use openai4rs::{OpenAI, chat_request, user};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();
    let model = "your model name";
    // Create a client with default settings
    let client = OpenAI::from_env()?;

    println!("Making API request with request-level HTTP settings...");

    // Create a chat completion with request-specific HTTP settings
    let messages = vec![user!("Hello, how are you?")];

    // Configure request-specific HTTP settings
    let request = chat_request(model, &messages)
        // Standard model parameters
        .temperature(0.7)
        .max_completion_tokens(150)
        // HTTP request configuration (these override client-level settings for this request only)
        .retry_count(2) // Use 2 retries for this request only
        .timeout_seconds(30) // 30 second timeout for this request only
        .user_agent("CustomApp/1.0 RequestExample".to_string()); // Custom User-Agent for this request

    let response = client.chat().create(request).await?;

    println!(
        "Response: {}",
        response.choices[0]
            .message
            .content
            .as_ref()
            .unwrap_or(&"No content".to_string())
    );

    Ok(())
}
