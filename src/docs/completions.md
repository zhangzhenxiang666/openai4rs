 返回对补全客户端的引用。

用于传统的文本补全（非聊天格式）。

# 示例

```rust,no_run
use openai4rs::{OpenAI, CompletionsParam};
use dotenvy::dotenv;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;
    let response = client
        .completions()
        .create(CompletionsParam::new("Qwen/Qwen3-235B-A22B-Instruct-2507", "Write a poem about the Rust programming language").max_tokens(64))
        .await;

    println!("Response: {:#?}", response);
    Ok(())
}
```
