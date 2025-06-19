use dotenvy::dotenv;
use futures::StreamExt;
use openai4rs::{error::OpenAIError, *};
use std::io;
use tokio_stream::wrappers::ReceiverStream;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();

    let model = "your model name";

    let mut input = String::new();

    let mut messages = vec![];
    loop {
        println!("\n# YOU\n");
        io::stdin().read_line(&mut input).unwrap();

        if input.contains("[QUIT]") {
            println!("bye");
            break;
        }

        messages.push(user!(input));

        let stream = client
            .chat()
            .create_stream(chat_request(model, &messages))
            .await
            .unwrap();

        messages.push(ChatCompletionMessageParam::assistant_from_str(
            process_stream(stream).await.as_str(),
        ))
    }
}

async fn process_stream(
    stream: ReceiverStream<Result<ChatCompletionChunk, OpenAIError>>,
) -> String {
    let mut stream = stream;
    let mut ai_output = String::new();
    let mut first = true;
    println!("\n# ASSISTANT\n");
    while let Some(result) = stream.next().await {
        let chunk = result.expect("Error processing stream");

        for choice in chunk.choices.iter() {
            if choice.delta.is_reasoning() && first {
                first = false;
                println!("## REASONING\n\n{}", choice.delta.get_reasoning_str())
            } else if choice.delta.is_reasoning() {
                print!("{}", choice.delta.get_reasoning_str());
            } else if !first {
                println!("\n\n## CONTENT\n\n");
                first = true;
            }
            if let Some(data) = choice.delta.content.as_ref() {
                print!("{}", data);
                ai_output.push_str(data);
            }
        }
    }
    println!("\n");
    ai_output
}
