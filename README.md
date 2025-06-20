# openai4rs

简体中文 | [English](README_en.md)

## 简介

openai4rs是一个非官方实现的基于`tokio`和`reqwest`的异步与大模型供应商以openai规范交互的rust实现的crate。

- chat
  - 流式响应 ✅
  - 工具调用 ✅
  - 思考模式 ✅
- completions
  - 非流式 ✅
  - 流式 ✅
- models
  - 模型列表 ✅
  - 获取单个模型 ✅

## 安装

- **使用cargo**

```bash
cargo add openai4rs
```

- **使用git**

```bash
git clone https://github.com/zhangzhenxiang666/openai4rs.git
```

## 使用指南

1. chat聊天

    - **非流式**

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

    - **流式**

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

    - **使用工具**

    ```rust
    use futures::StreamExt;
    use openai4rs::{ChatCompletionToolParam, OpenAI, chat_request, user};

    #[tokio::main]
    async fn main() {
        let base_url = "your base_url";
        let api_key = "your api_key";
        let client = OpenAI::new(api_key, base_url);
        // 定义工具
        let tools = vec![ChatCompletionToolParam::function(
            "get_time",
            "获取当前时间",
            serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        )];

        let messages = vec![user!("现在几点了?")];
        let mut stream = client
            .chat()
            .create_stream(
                chat_request("your model name", &messages)
                    .tools(tools.clone())
                    .tool_choice(openai4rs::ToolChoice::Auto), // 选择工具模式
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

    - **思考模型**

    模型必须支持思考功能, 例如**qwen**的**qwq-32b**

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

2. models 模型
    - **获取所有模型**

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

3. completions
    - **非流式**

    ```rust
    use openai4rs::{OpenAI, comletions_request};

    #[tokio::main]
    async fn main() {
        let base_url = "your base_url";
        let api_key = "your api_key";
        let client = OpenAI::new(api_key, base_url);
        let completion = client
            .completions()
            .create(comletions_request("your model name", "你好"))
            .await
            .unwrap();
        println!("{:#?}", completion)
    }
    ```

    - **流式**

    ```rust
    use futures::StreamExt;
    use openai4rs::{OpenAI, comletions_request};

    # [tokio::main]
    async fn main() {
        let base_url = "your base_url";
        let api_key = "your api_key";
        let client = OpenAI::new(api_key, base_url);
        let mut stream = client
            .completions()
            .create_stream(comletions_request("your model name", "你好"))
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
