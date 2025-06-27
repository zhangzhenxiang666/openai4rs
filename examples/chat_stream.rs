use dotenvy::dotenv;
use openai4rs::error::OpenAIError;
use openai4rs::{
    Apply, ChatCompletionChunk, ChatCompletionMessageParam, OpenAI, chat_request, user,
};
use std::io;
use tokio_stream::wrappers::ReceiverStream;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let mut input = String::new();

    let model = "your model name";

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
    let stream = stream;
    println!("\n# ASSISTANT\n");
    let ai_output = stream
        .apply_with_capture_async(String::new(), |capture, result| {
            Box::pin(async {
                let chunk = result.expect("Error processing stream");
                for choice in chunk.choices.iter() {
                    if let Some(data) = choice.delta.content.as_ref() {
                        print!("{}", data);
                        capture.push_str(data);
                    }
                }
            })
        })
        .await;
    println!("\n");
    ai_output
}
