use dotenvy::dotenv;
use futures::StreamExt;
use openai4rs::*;
use std::io::{Write, stdin, stdout};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let model = "Qwen/Qwen3-235B-A22B-Instruct-2507";
    let mut messages = vec![system!(content: "You are a helpful assistant.")];

    loop {
        print!("You: ");
        stdout().flush()?;
        let mut user_input = String::new();
        stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        if user_input.eq_ignore_ascii_case("exit") {
            println!("Goodbye!");
            break;
        }

        messages.push(user!(content: user_input));

        let request = ChatParam::new(model, &messages);

        let mut stream = client.chat().create_stream(request).await?;
        let mut first_content = true;

        let mut assistant_message = String::new();

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    if chunk.has_content() {
                        if first_content {
                            print!("Assistant: ");
                            first_content = false;
                        }
                        if let Some(content) = chunk.content() {
                            print!("{}", content);
                            assistant_message.push_str(content);
                            std::io::stdout().flush()?;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("\nAn error occurred during streaming: {}", e);
                    break;
                }
            }
        }
        if !assistant_message.is_empty() {
            messages.push(assistant!(assistant_message));
        }
        println!();
    }

    Ok(())
}
