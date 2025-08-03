use dotenvy::dotenv;
use openai4rs::{OpenAI, chat_request, user};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv().ok();
    let model = "your model name";
    // Create a client with custom HTTP settings
    let client = OpenAI::from_env()?;

    // Update the client configuration
    client
        .update_config(|config| {
            config
                .set_retry_count(3) // Set max 3 retry attempts
                .set_timeout_seconds(120) // Set 120 second timeout
                .set_connect_timeout_seconds(5) // Set 5 second connect timeout
                .set_user_agent(Some("MyApp/1.0".to_string())); // Set custom user agent

            // Uncomment to use a proxy
            // config.set_proxy(Some("http://localhost:8080".to_string()));
        })
        .await;

    println!("Making API request with custom HTTP settings...");

    // Create a chat completion
    let messages = vec![user!("Hello, how are you?")];
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
