use futures::StreamExt;
use openai4rs::{
    ChatCompletionAssistantMessageParam, ChatCompletionMessageParam,
    ChatCompletionMessageToolCallParam, ChatCompletionToolParam, OpenAI, chat_request, tool, user,
};

use dotenvy::dotenv;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();

    let model = "your model name";

    let assistant_tool =
        ChatCompletionMessageParam::Assistant(ChatCompletionAssistantMessageParam {
            name: None,
            content: None,
            refusal: None,
            tool_calls: Some(vec![ChatCompletionMessageToolCallParam::function(
                "1",
                "get_location_time",
                "{location:'BeiJing'}",
            )]),
        });
    let messages = vec![
        user!("北京现在几点了?"),
        assistant_tool,
        tool!("1", "现在是北京时间中午12点32分"),
    ];
    let tools = vec![ChatCompletionToolParam::function(
        "get_location_time",
        "Get current time for a specified location",
        serde_json::json!({
            "type": "object",
            "properties": {
                "location": {
                    "type": "string",
                    "description": "The name of the city or geographical location to get the current time for."
                }
            },
            "required": ["location"]
        }),
    )];

    let mut stream = client
        .chat()
        .create_stream(chat_request(model, &messages).tools(tools.clone()))
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
