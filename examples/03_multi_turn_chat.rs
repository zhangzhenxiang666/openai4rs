use dotenvy::dotenv;
use openai4rs::*;
use std::io::{Write, stdin, stdout};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let model = "Qwen/Qwen3-Coder-480B-A35B-Instruct";
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

        let request = chat_request(model, &messages);

        let response = client.chat().create(request).await?;
        if let Some(content) = response.content() {
            println!("Assistant: {}\n", content);
            messages.push(assistant!(content));
        } else {
            println!("Assistant: No response.\n");
        }
    }

    Ok(())
}
