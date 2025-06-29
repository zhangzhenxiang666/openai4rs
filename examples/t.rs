use futures::stream::StreamExt;
use openai4rs::{OpenAI, StreamChoice, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let mut messages = vec![user!("请详细介绍一下 Rust 的所有权机制")];

    let mut stream = client
        .chat()
        .create_stream(chat_request("your_model_name", &messages))
        .await
        .unwrap();

    let mut merged_choice: Option<StreamChoice> = None;
    while let Some(result) = stream.next().await {
        let chat_completion_chunk = result.unwrap();
        let choice = chat_completion_chunk.choices[0].clone();
        merged_choice = Some(match merged_choice {
            Some(l) => l + choice,
            None => choice,
        })
    }
    messages.push(merged_choice.unwrap().delta.into());

    messages.push(user!("好的谢谢你"));

    let chat_completion = client
        .chat()
        .create(chat_request("your_model_name", &messages))
        .await
        .unwrap();

    messages.push(chat_completion.choices[0].message.clone().into())
}
