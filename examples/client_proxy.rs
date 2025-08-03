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

    // Create a custom configuration with proxy settings
    let mut config = Config::new(api_key, base_url);

    // Set proxy (this will be used for all requests made by this client)
    // You can use HTTP, HTTPS or SOCKS5 proxies
    // Example: "http://localhost:8080" or "socks5://user:pass@127.0.0.1:1080"
    config.set_proxy(Some("http://localhost:8080".to_string()));

    // Configure other HTTP settings
    config
        .set_retry_count(3)
        .set_timeout_seconds(60)
        .set_user_agent(Some("MyProxyApp/1.0".to_string()));

    // Create client with custom configuration
    let client = OpenAI::with_config(config);

    println!("Making API request through proxy...");

    // Create a chat completion
    let messages = vec![user!("What's my IP address?")];
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
