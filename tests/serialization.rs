use openai4rs::*;
use std::fs;

#[test]
fn test_deserialize_chatcompletion() {
    let json = fs::read_to_string("./assets/chatcompletion.json").unwrap();
    let chatcompletion: Result<ChatCompletion, _> = serde_json::from_str(json.as_str());
    assert!(chatcompletion.is_ok());

    let chatcompletion = chatcompletion.unwrap();
    assert_eq!(chatcompletion.id, "chatcmpl-abc123");
    assert_eq!(chatcompletion.choices.len(), 1);
    let choice = &chatcompletion.choices[0];
    assert_eq!(choice.index, 0);
    assert!(matches!(choice.finish_reason, FinishReason::ToolCalls));
    assert_eq!(choice.message.role, "assistant");
    assert_eq!(choice.message.content.as_deref(), None);
}

#[test]
fn test_deserialize_chatcompletion_stream() {
    let json = fs::read_to_string("./assets/chatcompletionchunk.json").unwrap();
    let chatcompletion_chunk: Result<ChatCompletionChunk, _> = serde_json::from_str(json.as_str());
    assert!(chatcompletion_chunk.is_ok());

    let chunk = chatcompletion_chunk.unwrap();
    assert_eq!(chunk.id, "chatcmpl-abc123");
    assert_eq!(chunk.choices.len(), 1);
    let choice = &chunk.choices[0];
    assert_eq!(choice.index, 0);
    assert_eq!(choice.delta.content.as_deref(), None);
}

#[test]
fn test_deserialize_chat_completion_tool_param() {
    // 检查反序列化是否正确
    let json = fs::read_to_string("./assets/chat_completion_tool_param.json").unwrap();
    let chat_completion_tool_param: ChatCompletionToolParam =
        serde_json::from_str(json.as_str()).unwrap();

    // 验证解析数据
    let ChatCompletionToolParam::Function(function_def) = chat_completion_tool_param;

    assert_eq!(function_def.name, "get_current_weather");
    assert_eq!(
        function_def.description,
        "Get the current weather in a given location"
    );

    // 检查直接的 FunctionDefinition 格式
    let json = fs::read_to_string("./assets/function_definition.json").unwrap();
    let chat_completion_tool_param: ChatCompletionToolParam =
        serde_json::from_str(json.as_str()).unwrap();

    // 验证解析数据
    let ChatCompletionToolParam::Function(function_def) = chat_completion_tool_param;

    assert_eq!(function_def.name, "get_current_weather");
    assert_eq!(
        function_def.description,
        "Get the current weather in a given location"
    );
}

#[test]
fn test_assistant_serialize() {
    let assistant = assistant!(
        content = "content",
        name = "name",
        refusal = "refusal",
        tool_calls = vec![ChatCompletionMessageToolCallParam::function(
            "id",
            "name",
            "{'path': '/.cargo'}",
        )]
    );

    let left = serde_json::to_value(&assistant).unwrap();
    let right: serde_json::Value = serde_json::json!({
        "content": "content",
        "name": "name",
        "refusal": "refusal",
        "role": "assistant",
        "tool_calls": [
            {
                "function": {
                    "arguments": "{'path': '/.cargo'}",
                    "name": "name"
                },
                "id": "id",
                "type": "function"
            }
        ]
    });
    assert_eq!(left, right);
}

#[test]
fn test_chat_completion_helpers() {
    let message = ChatCompletionMessage {
        role: "assistant".to_string(),
        content: Some("Hello, world!".to_string()),
        refusal: None,
        reasoning: None,
        annotations: None,
        tool_calls: Some(vec![ChatCompletionToolCall {
            index: 0,
            function: Function {
                id: "id".to_string(),
                name: "get_current_weather".to_string(),
                arguments: "{'path': '/.cargo'}".to_string(),
            },
            r#type: "function".to_string(),
        }]),
        extra_fields: None,
    };

    let choice = FinalChoice {
        index: 0,
        finish_reason: FinishReason::Stop,
        message: message.clone(),
        logprobs: None,
    };

    let chat_completion = ChatCompletion {
        id: "chatcmpl-123".to_string(),
        choices: vec![choice],
        created: 1234567890,
        model: "gpt-3.5-turbo".to_string(),
        object: "chat.completion".to_string(),
        usage: None,
        service_tier: None,
        system_fingerprint: None,
        extra_fields: None,
    };

    // 验证数据
    assert_eq!(chat_completion.content(), Some("Hello, world!"));
    assert!(chat_completion.has_tool_calls());

    let tool_calls = chat_completion.tool_calls().unwrap();
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0].function.name, "get_current_weather");
}

#[test]
fn test_chat_completion_chunk_helpers() {
    let delta = ChoiceDelta {
        content: Some("Hello, world!".to_string()),
        refusal: None,
        reasoning: None,
        role: Some("assistant".to_string()),
        tool_calls: Some(vec![ChatCompletionToolCall {
            index: 0,
            function: Function {
                id: "id".to_string(),
                name: "get_current_weather".to_string(),
                arguments: "{'path': '/.cargo'}".to_string(),
            },
            r#type: "function".to_string(),
        }]),
        extra_fields: None,
    };

    let choice = StreamChoice {
        index: 0,
        delta: delta.clone(),
        finish_reason: Some(FinishReason::Stop),
        logprobs: None,
    };

    let chat_completion_chunk = ChatCompletionChunk {
        id: "chatcmpl-123".to_string(),
        choices: vec![choice],
        created: 1234567890,
        model: "gpt-3.5-turbo".to_string(),
        object: "chat.completion.chunk".to_string(),
        usage: None,
        service_tier: None,
        system_fingerprint: None,
        extra_fields: None,
    };

    // 验证数据
    assert_eq!(chat_completion_chunk.content(), Some("Hello, world!"));
    let tool_calls = chat_completion_chunk.tool_calls().unwrap();
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0].function.name, "get_current_weather");
    let deltas: Vec<&ChoiceDelta> = chat_completion_chunk.deltas().collect();
    assert_eq!(deltas.len(), 1);
}

#[test]
fn test_chat_completion_missing_id() {
    let json = serde_json::json!({
        "object": "chat.completion",
        "created": 1699896916,
        "model": "gpt-4o-mini",
        "choices": [
            {
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [
                        {
                            "id": "call_abc123",
                            "type": "function",
                            "function": {
                                "name": "get_current_weather",
                                "arguments": "{\n\"location\": \"Boston, MA\"\n}"
                            }
                        }
                    ]
                },
                "logprobs": null,
                "finish_reason": "tool_calls"
            }
        ],
        "usage": {
            "prompt_tokens": 82,
            "completion_tokens": 17,
            "total_tokens": 99,
            "completion_tokens_details": {
                "reasoning_tokens": 0,
                "accepted_prediction_tokens": 0,
                "rejected_prediction_tokens": 0
            }
        }
    });

    let chatcompletion: Result<ChatCompletion, _> = serde_json::from_value(json);
    assert!(chatcompletion.is_ok());

    let chatcompletion = chatcompletion.unwrap();
    assert_eq!(chatcompletion.id, "0");
    assert_eq!(chatcompletion.choices.len(), 1);
    let choice = &chatcompletion.choices[0];
    assert_eq!(choice.index, 0);
    assert!(matches!(choice.finish_reason, FinishReason::ToolCalls));
    assert_eq!(choice.message.role, "assistant");
    assert_eq!(choice.message.content.as_deref(), None);
}
