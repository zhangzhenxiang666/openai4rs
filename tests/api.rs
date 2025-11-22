use dotenvy::dotenv;
use openai4rs::*;

const MODEL_NAME: &str = "Qwen/Qwen3-235B-A22B-Instruct-2507";

#[tokio::test]
async fn test_chat() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let messages = vec![user!("Hello")];
    let mut retries = 3;
    while retries > 0 {
        let request = ChatParam::new(MODEL_NAME, &messages).temperature(0.0);
        match client.chat().create(request).await {
            Ok(result) => {
                assert!(result.has_content());
                return;
            }
            Err(e) if e.is_retryable() => {
                retries -= 1;
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            }
            Err(e) => {
                panic!("Non-retryable error: {:#?}", e);
            }
        }
    }
    panic!("Test failed after multiple retries");
}

#[tokio::test]
async fn test_chat_tool() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let messages = vec![user!("现在上海几点")];
    let parameters = Parameters::object()
        .property(
            "uct",
            Parameters::string()
                .description("时区, UTC±HH:MM格式")
                .build(),
        )
        .require("uct")
        .build()
        .unwrap();
    let tools = vec![ChatCompletionToolParam::function(
        "get_current_time",
        "根据时区获取当前时间",
        parameters,
    )];
    let request = ChatParam::new(MODEL_NAME, &messages)
        .tools(tools)
        .tool_choice(ToolChoice::Required);
    let result = client.chat().create(request).await;
    assert!(result.is_ok());
    let ok = result.unwrap();
    assert!(ok.has_tool_calls());
    let tool_req = ok.tool_calls().unwrap().first().unwrap();
    assert_eq!("get_current_time", tool_req.function.name.as_str());
}

#[tokio::test]
async fn test_openai_error_authentication() {
    let base_url = "https://openrouter.ai/api/v1";
    let api_key = "******";
    let client = OpenAI::new(api_key, base_url);
    let messages = vec![user!("hello world")];
    let result = client
        .chat()
        .create(
            ChatParam::new(MODEL_NAME, &messages)
                .temperature(0.0)
                .max_completion_tokens(512),
        )
        .await;
    match result {
        Ok(_) => panic!("Unexpected success response"),
        Err(err) => {
            if !err.is_authentication() {
                panic!("Unexpected error: {:#?}", err);
            }
        }
    }
}

#[tokio::test]
async fn test_openai_error_bad_request() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let messages = vec![user!("hello world")];
    let result = client
        .chat()
        .create(
            ChatParam::new("invalid-model-name", &messages)
                .temperature(0.0)
                .max_completion_tokens(512),
        )
        .await;
    match result {
        Ok(_) => panic!("Unexpected success response"),
        Err(err) => {
            if !err.is_bad_request() {
                panic!("Unexpected error: {:#?}", err);
            }
        }
    }
}

#[tokio::test]
async fn test_models_list() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let models = client.models().list(Default::default()).await;
    assert!(models.is_ok())
}

#[tokio::test]
async fn test_embeddings() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let embeddings = client
        .embeddings()
        .create(EmbeddingsParam::new(
            "Qwen/Qwen3-Embedding-0.6B",
            "hello world",
        ))
        .await;
    assert!(embeddings.is_ok());
}

#[tokio::test]
async fn test_embedddings_with_encoding_format() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let embeddings = client
        .embeddings()
        .create(
            EmbeddingsParam::new("Qwen/Qwen3-Embedding-0.6B", "hello world")
                .encoding_format(EncodingFormat::Base64),
        )
        .await;

    assert!(embeddings.is_ok());

    let embeddings = embeddings.unwrap();

    for embedding in embeddings.embeddings() {
        assert!(embedding.as_base64().is_some());
        let vector = embedding.vector();
        assert!(vector.is_some());
        let vector = vector.unwrap();
        assert!(vector.len() > 0);
    }
}
