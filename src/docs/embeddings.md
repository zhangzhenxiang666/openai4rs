返回对嵌入客户端的引用。

使用此客户端为搜索、聚类和其他机器学习任务生成文本的向量表示。

# 示例

## 基本嵌入生成

```rust,no_run
use openai4rs::*;
use dotenvy::dotenv;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let response = client
        .embeddings()
        .create(EmbeddingsParam::new("text-embedding-ada-002", "Hello, world!"))
        .await?;

    println!("Generated {} embeddings", response.len());
    println!("Total tokens used: {}", response.total_tokens());
    Ok(())
}
```

## 多文本嵌入

```rust,no_run
use openai4rs::*;
use dotenvy::dotenv;
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;
    let texts = vec!["Hello, world!", "How are you?", "Rust is awesome!"];

    let response = client
        .embeddings()
        .create(EmbeddingsParam::new("text-embedding-ada-002", texts))
        .await?;

    println!("Generated {} embeddings", response.len());
    for (i, embedding) in response.embeddings().iter().enumerate() {
        println!("Embedding {}: {} dimensions", i, embedding.dimensions());
    }
    Ok(())
}
```
