use std::time::Duration;

use dotenvy::dotenv;
use openai4rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Get the API key from the environment
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let base_url = std::env::var("OPENAI_BASE_URL")?;
    // 1. Basic client with default settings
    let basic_client = OpenAI::new(&api_key, &base_url);

    // 2. Client with a custom base URL (e.g., for a proxy or a different provider)
    let _custom_base_url_client = Config::builder()
        .api_key(api_key.clone())
        .base_url(base_url.clone()) // Replace with your custom base URL
        .build_openai()?;

    // 3. Client with a proxy
    let proxy_config = Config::builder()
        .api_key(api_key.clone())
        .base_url(base_url.clone())
        .proxy("http://proxy.example.com:8080")
        .build()?;
    let _proxy_client = OpenAI::with_config(proxy_config);

    // 4. Client with custom timeout
    let timeout_config = Config::builder()
        .api_key(api_key.clone())
        .base_url(base_url.clone())
        .timeout(Duration::from_secs(120))
        .build()?;
    let _timeout_client = OpenAI::with_config(timeout_config);

    // 5. Add global request headers, query parameters, and request body
    let _global_http_params_client = Config::builder()
        .api_key(api_key.clone())
        .base_url(base_url.clone())
        .header(
            "test-header",
            header::HeaderValue::from_str("test-valeu").unwrap(),
        )
        .body("global_body_key", "global_body_value")
        .build()?;
    let _global_http_params_client = OpenAI::with_config(_global_http_params_client);

    // For demonstration, we'll use the basic client to make a simple request.
    // In a real application, you would use the client that best fits your needs.

    let model = "Qwen/Qwen3-235B-A22B-Instruct-2507";
    let messages = vec![user!(content: "Ping to check if the client is working.")];
    let request = ChatParam::new(model, &messages);

    println!("Testing basic client...");
    match basic_client.chat().create(request).await {
        Ok(response) => {
            if let Some(content) = response.content() {
                println!("Success: {}", content);
            }
        }
        Err(e) => {
            eprintln!("Error with basic client: {}", e);
        }
    }

    Ok(())
}
