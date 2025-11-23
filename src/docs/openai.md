用于与OpenAI兼容的端点API交互的OpenAI客户端

这是主要的客户端结构体，提供对聊天补全、文本补全和模型列表等端点的访问。

它使用async/await进行非阻塞操作并支持流式响应。

所有的端点都提供了对应的参数构建器, 例如ChatParam

# Features

- **chat端点**: 主流的聊天补全API(对于推理做了特殊处理, 将reasoning(openrouter返回的推理字段)和reasoning_content(openai官方返回的推理字段)都统一映射到reasoning字段)
- **completions端点**: 支持传统的文本补全API(建议使用chat端点)
- **models端点**: 列出和检索模型信息
- **embeddings端点**: 嵌入API
- **宏**: 定义了user, assistant, system, tool等宏简化消息列表的构建

# Examples

## chat端点

```rust,no_run
use dotenvy::dotenv;
use openai4rs::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();

    let model = "gpt-5.1";
    let messages = vec![system!("你是一个ai助手, 善于回答用户的问题"), user!("你好")];

    let req = ChatParam::new(model, &messages).max_completion_tokens(1024);

    let result = client.chat().create(req).await;
    assert!(result.is_ok());
    let res = result.unwrap();
    println!("content: {:#?}", res.content());
}
```

## completions端点(建议使用chat端点)

```rust,no_run
use dotenvy::dotenv;
use openai4rs::*;
#[tokio::main]
async fn main() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let model = "gpt-5.1";
    let prompt = "你好";
    let req = CompletionsParam::new(model, prompt).temperature(0.);
    let result = client.completions().create(req).await;
    assert!(result.is_ok());
    let res = result.unwrap();
    println!("res: {:#?}", res);
}

```

## models端点

```rust,no_run
use dotenvy::dotenv;
use openai4rs::*;
#[tokio::main]
async fn main() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let result = client.models().list(Default::default()).await;
    assert!(result.is_ok());
    let res = result.unwrap();
    res.data.iter().for_each(|model| {
        println!("{}", model.id);
    });
}
```

## embeddings端点

```rust,no_run
use dotenvy::dotenv;
use openai4rs::*;
#[tokio::main]
async fn main() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let model = "text-embedding-ada-002";
    let input = "hello world";
    let req = EmbeddingsParam::new(model, input);
    let result = client.embeddings().create(req).await;
    assert!(result.is_ok());
    let res = result.unwrap();
    res.embedding_vectors_decoded().iter().for_each(|v| {
        print!("dim={} ,", v.len());
    });
}
```
