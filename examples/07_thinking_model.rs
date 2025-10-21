use std::io::{self, Write};

use dotenvy::dotenv;
use futures::StreamExt;
use openai4rs::*;

/// This example demonstrates how to send a request to a model that supports complex reasoning ("thinking").
/// Note: Not all models can handle tasks that require deep thinking well.
/// Please ensure that the model you choose has strong logical reasoning and planning capabilities.
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    // Select a model with strong reasoning capabilities.
    // The model name here is just an example; please replace it with the advanced model you are actually using.
    let model = "Qwen/Qwen3-235B-A22B-Thinking-2507";

    let messages = vec![
        system!(content: "You are a master of logic and reasoning. When you are asked a question, you should think step by step, and then give the final answer."),
        user!(content: "There are three friends: Tom, Mary, and John. They are a teacher, a doctor, and an engineer. Tom is not the teacher. Mary is neither the teacher nor the doctor. Based on this information, determine each person's profession."),
    ];

    let request = chat_request(model, &messages).body("enable_thinking", true);

    println!("Sending request to model: {}...", model);
    println!("This may take a moment as the model 'thinks' through the logic puzzle...");

    let mut stream = client.chat().create_stream(request).await?;
    let mut first_reasoning = true;
    let mut first_content = true;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if chunk.has_reasoning() {
                    if first_reasoning {
                        println!("\n\n========Think========");
                        first_reasoning = false;
                    }
                    if let Some(reasoning) = chunk.reasoning() {
                        print!("{}", reasoning);
                        io::stdout().flush()?;
                    }
                }

                if chunk.has_content() {
                    if first_content {
                        println!("\n\n========Answer========");
                        first_content = false;
                    }
                    if let Some(content) = chunk.content() {
                        print!("{}", content);
                        io::stdout().flush()?;
                    }
                }
            }
            Err(e) => {
                eprintln!("\n========Error========");
                eprintln!("An error occurred during streaming: {}", e);
                break;
            }
        }
    }
    println!();

    Ok(())
}
