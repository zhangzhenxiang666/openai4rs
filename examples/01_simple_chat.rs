use dotenvy::dotenv;
use openai4rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let model = "Qwen/Qwen3-235B-A22B-Instruct-2507";
    let messages = vec![
        system!("You are a helpful assistant."),
        user!("Introduce the Rust programming language in one sentence."),
    ];

    let request = ChatParam::new(model, &messages);

    println!("Sending request to model: {}...", model);

    let response = client.chat().create(request).await?;

    if let Some(content) = response.content() {
        println!("\nResponse:\n{}", content);
    } else {
        println!("\nNo content in response.");
    }

    Ok(())
}
