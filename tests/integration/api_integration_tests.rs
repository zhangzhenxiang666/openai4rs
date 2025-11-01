use dotenvy::dotenv;
use openai4rs::{OpenAI, chat::*, embeddings_request, models_request, user};

const MODEL_NAME: &str = "Qwen/Qwen3-235B-A22B-Instruct-2507";

#[tokio::test]
async fn test_chat() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let messages = vec![user!("Hello")];
    let mut retries = 3;
    while retries > 0 {
        let request = chat_request(MODEL_NAME, &messages).temperature(0.0);
        match client.chat().create(request).await {
            Ok(result) => {
                assert!(
                    result
                        .choices
                        .first()
                        .is_some_and(|choice| choice.message.content.is_some())
                );
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
async fn test_openai_error_authentication() {
    let base_url = "https://openrouter.ai/api/v1";
    let api_key = "******";
    let client = OpenAI::new(api_key, base_url);
    let messages = vec![user!("hello world")];
    let result = client
        .chat()
        .create(
            chat_request(MODEL_NAME, &messages)
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
async fn test_models_list() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let models = client.models().list(models_request()).await;
    assert!(models.is_ok())
}

#[tokio::test]
async fn test_embeddings() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let embeddings = client
        .embeddings()
        .create(embeddings_request(
            "Qwen/Qwen3-Embedding-0.6B",
            "hello world",
        ))
        .await;
    assert!(embeddings.is_ok());
}
