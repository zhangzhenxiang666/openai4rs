# openai4rs

[简体中文](README.md) | English

## Introduction

openai4rs is an unofficial Rust crate implementation based on `tokio` and `reqwest` for asynchronous interaction with large language model providers following the OpenAI specification.

- chat
  - Streaming response ✅
  - Tool calling ✅
  - Reasoning mode ✅
- completions
  - Non-streaming ✅
  - Streaming ✅
- models
  - List models ✅
  - Retrieve single model ✅

## Installation

- **Using cargo**

```bash
cargo add openai4rs
```

- **Using git**

```bash
git clone https://github.com/zhangzhenxiang666/openai4rs.git
```

## Usage Guide

1. Chat functionality

    - **Non-streaming**

    ```rust
    use openai4rs::{OpenAI, chat_request, user};

    #[tokio::main]
    async fn main() {
        let base_url = "your base_url";
        let api_key = "your api_key";
        let client = OpenAI::new(api_key, base_url);
        let messages = vec![user!("hello")];
        let chat_completion = client
            .chat()
            .create(chat_request("your model name", &messages))
            .await
            .unwrap();
        println!("{:#?}", chat_completion);
    }

    ```

    - **Streaming**

    ```rust
    use futures::StreamExt;
    use openai4rs::{OpenAI, chat_request, user};

    #[tokio::main]
    async fn main() {
        let base_url = "your base_url";
        let api_key = "your api_key";
        let client = OpenAI::new(api_key, base_url);
        let messages = vec![user!("hello")];
        let mut stream = client
            .chat()
            .create_stream(chat_request("your model name", &messages))
            .await
            .unwrap();
        while let Some(result) = stream.next().await {
            let chat_completion_chunk = result.unwrap();
            println!("{:#?}", chat_completion_chunk);
        }
    }
    ```

    - **Using Tools**

    ```rust
    use futures::StreamExt;
    use openai4rs::{ChatCompletionToolParam, OpenAI, chat_request, user};

    #[tokio::main]
    async fn main() {
        let base_url = "your base_url";
        let api_key = "your api_key";
        let client = OpenAI::new(api_key, base_url);
        // Define tools
        let tools = vec![ChatCompletionToolParam::function(
            "get_time",
            "Get current time",
            serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        )];

        let messages = vec![user!("What time is it now?")];
        let mut stream = client
            .chat()
            .create_stream(
                chat_request("your model name", &messages)
                    .tools(tools.clone())
                    .tool_choice(openai4rs::ToolChoice::Auto), // Choose tool mode
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
    ```

    - **Reasoning Models**

    The model must support reasoning functionality, for example, **qwen**'s **qwq-32b**

    ```rust
    use futures::StreamExt;
    use openai4rs::{OpenAI, chat_request, user};

    #[tokio::main]
    async fn main() {
        let base_url = "your base_url";
        let api_key = "your api_key";
        let client = OpenAI::new(api_key, base_url);
        let messages = vec![user!("hello")];
        let mut stream = client
            .chat()
            .create_stream(chat_request("your model name", &messages))
            .await
            .unwrap();
        while let Some(result) = stream.next().await {
            let chat_completion_chunk = result.unwrap();
            for choice in chat_completion_chunk.choices.iter() {
                if choice.delta.is_reasoning() {
                    println!("## REASONING\n{}", choice.delta.get_reasoning_str())
                }
                if let Some(content) = &choice.delta.content {
                    if !content.is_empty() {
                        println!("## CONTENT\n{}", content);
                    }
                }
            }
        }
    }
    ```

2. Models functionality
    - **Get all models**

    ```rust
    use openai4rs::{OpenAI, models_request};

    #[tokio::main]
    async fn main() {
        let base_url = "your base_url";
        let api_key = "your api_key";
        let client = OpenAI::new(api_key, base_url);
        let models = client.models().list(models_request()).await.unwrap();
        println!("{:#?}", models);
    }
    ```

3. Completions functionality
    - **Non-streaming**

    ```rust
    use openai4rs::{OpenAI, comletions_request};

    #[tokio::main]
    async fn main() {
        let base_url = "your base_url";
        let api_key = "your api_key";
        let client = OpenAI::new(api_key, base_url);
        let completion = client
            .completions()
            .create(comletions_request("your model name", "Hello"))
            .await
            .unwrap();
        println!("{:#?}", completion)
    }
    ```

    - **Streaming**

    ```rust
    use futures::StreamExt;
    use openai4rs::{OpenAI, comletions_request};

    #[tokio::main]
    async fn main() {
        let base_url = "your base_url";
        let api_key = "your api_key";
        let client = OpenAI::new(api_key, base_url);
        let mut stream = client
            .completions()
            .create_stream(comletions_request("your model name", "Hello"))
            .await
            .unwrap();
        while let Some(result) = stream.next().await {
            match result {
                Ok(completion) => println!("{:#?}", completion),
                Err(err) => println!("{}", err),
            }
        }
    }
    ```
