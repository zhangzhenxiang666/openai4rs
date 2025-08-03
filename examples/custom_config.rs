use dotenvy::dotenv;
use openai4rs::{Config, OpenAI, chat_request, user};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();
    let model = "your model name";
    // Get API key from environment
    let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

    // Create a custom configuration
    let mut config = Config::new(api_key, base_url);

    // Configure HTTP settings
    config
        .set_retry_count(2) // Set max 2 retry attempts
        .set_timeout_seconds(180) // Set 3 minute timeout
        .set_connect_timeout_seconds(15) // Set 15 second connect timeout
        .set_user_agent(Some("CustomApp/2.0".to_string())); // Set custom user agent

    // Create client with custom configuration
    let client = OpenAI::with_config(config);

    println!("Making API request with custom configuration...");

    // Create a chat completion
    let messages = vec![user!(
        "What are the benefits of configuring HTTP request parameters?"
    )];

    let response = client.chat().create(chat_request(model, &messages)).await?;

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
