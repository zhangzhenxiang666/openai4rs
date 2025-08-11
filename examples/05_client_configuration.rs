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
    let _proxy_client = Config::builder()
        .api_key(api_key.clone())
        .base_url(base_url.clone())
        .proxy("http://proxy.example.com:8080".to_string()) // Replace with your proxy URL
        .build_openai()?;

    // 4. Client with custom timeout
    let _timeout_client = Config::builder()
        .api_key(api_key)
        .base_url(base_url.clone())
        .timeout_seconds(120) // 2 minutes
        .build_openai()?;

    // For demonstration, we'll use the basic client to make a simple request.
    // In a real application, you would use the client that best fits your needs.

    let model = "Qwen/Qwen3-Coder-480B-A35B-Instruct";
    let messages = vec![user!(content: "Ping to check if the client is working.")];
    let request = chat_request(model, &messages);

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
