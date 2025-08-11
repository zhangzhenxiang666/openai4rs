use dotenvy::dotenv;
use openai4rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let model = "Qwen/Qwen3-Coder-480B-A35B-Instruct";
    let messages = vec![
        system!(content: "You are a helpful assistant."),
        user!(content: "Introduce the Rust programming language in one sentence."),
    ];

    let request = chat_request(model, &messages);

    println!("Sending request to model: {}...", model);

    let response = client.chat().create(request).await?;

    if let Some(content) = response.content() {
        println!("\nResponse:\n{}", content);
    } else {
        println!("\nNo content in response.");
    }

    Ok(())
}
