返回对聊天补全客户端的引用。

使用此客户端执行聊天补全，包括流式响应、工具调用和推理模式交互。

# 示例

## 基本聊天补全

```rust,no_run
use openai4rs::*;
use dotenvy::dotenv;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;
    let messages = vec![user!("Hello, how are you?")];
   
    let response = client
        .chat()
        .create(ChatParam::new("Qwen/Qwen3-235B-A22B-Instruct-2507", &messages))
        .await?;
   
    println!("Response: {:#?}", response);
    Ok(())
}
```

## 流式聊天补全

```rust,no_run
use futures::StreamExt;
use openai4rs::*;
use dotenvy::dotenv;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;
    let messages = vec![user!("Tell me a story")];
   
    let mut stream = client
        .chat()
        .create_stream(
            ChatParam::new("Qwen/Qwen3-235B-A22B-Instruct-2507", &messages)
                .max_completion_tokens(64),
        )
        .await?;
   
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        if let Some(choice) = chunk.choices.first() {
            if let Some(content) = &choice.delta.content {
                print!("{}", content);
            }
        }
    }
    Ok(())
}
```
