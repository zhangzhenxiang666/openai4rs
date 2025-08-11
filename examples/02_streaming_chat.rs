use std::io::{self, Write};

use dotenvy::dotenv;
use futures::StreamExt;
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

    let request = chat_request(model, &messages).build()?;

    println!("Sending request to model: {}...", model);

    let mut stream = client.chat().create_stream(request).await?;
    let mut first_content = true;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if chunk.has_content() {
                    if first_content {
                        println!("\n========Response========");
                        first_content = false;
                    }
                    if let Some(content) = chunk.content() {
                        print!("{}", content);
                        io::stdout().flush()?;
                    }
                }
            }
            Err(e) => {
                eprintln!("\nAn error occurred during streaming: {}", e);
                break;
            }
        }
    }
    println!();

    Ok(())
}
