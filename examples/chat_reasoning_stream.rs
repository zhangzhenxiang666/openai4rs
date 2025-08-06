use dotenvy::dotenv;
use openai4rs::{
    Apply, ChatCompletionChunk, ChatCompletionMessageParam, OpenAI, chat_request,
    error::OpenAIError, user,
};
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
    println!("\n# ASSISTANT\n");
    let (ai_output, ..) = stream
        .fold_async((String::new(), true), |capture, result| {
            Box::pin(async move {
                let chunk = result.expect("Error processing stream");
                let (mut ai_output, mut first) = capture;
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
                (ai_output, first)
            })
        })
        .await;
    println!("\n");
    ai_output
}
