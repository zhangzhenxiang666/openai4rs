 返回对模型客户端的引用。

用于列出可用模型或检索模型信息。

# 示例

```rust
use openai4rs::OpenAI;
use dotenvy::dotenv;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;
    // 列出所有可用模型
    let models = client
        .models()
        .list(Default::default())
        .await?;

    for model in models.data {
        println!("Model: {}", model.id);
    }
    Ok(())
}
```
