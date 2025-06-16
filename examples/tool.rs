use futures::StreamExt;
use openai4rs::{ChatCompletionToolParam, OpenAI, chat_request, user};

use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let messages = vec![user!("现在几点了?")];
    let tools = vec![ChatCompletionToolParam::function(
        "get_time",
        "Get current time",
        serde_json::json!({
            "type": "object",
            "properties": {}
        }),
    )];

    let mut stream = client
        .chat()
        .create_stream(
            chat_request("meta-llama/llama-3.3-70b-instruct:free", &messages)
                .tools(tools.clone())
                .tool_choice(openai4rs::ToolChoice::Auto),
        )
        .await
        .unwrap();

    while let Some(result) = stream.next().await {
        match result {
            Ok(chunk) => {
                println!("{:#?}", chunk);
            }
            Err(err) => {
                println!("{:#?}", err);
            }
        }
    }
}
