# openai4rs

[![Crates.io](https://img.shields.io/crates/v/openai4rs)](https://crates.io/crates/openai4rs)
[![Documentation](https://docs.rs/openai4rs/badge.svg)](https://docs.rs/openai4rs)
[![License](https://img.shields.io/crates/l/openai4rs)](LICENSE)

简体中文 | [English](README_en.md)

一个基于 `tokio` 和 `reqwest` 的异步 Rust crate，用于与遵循 OpenAI 规范的大模型供应商进行交互。

## ✨ 特性

### 🗨️ Chat 聊天

- ✅ 流式响应
- ✅ 工具调用
- ✅ 思考模式

### 📝 Completions 文本补全

- ✅ 非流式响应
- ✅ 流式响应

### 🤖 Models 模型管理

- ✅ 获取模型列表
- ✅ 获取单个模型信息

### 🔄 HTTP 请求控制

- ✅ 可配置的重试次数
- ✅ 可配置的请求超时
- ✅ 可配置的连接超时
- ✅ HTTP 代理支持
- ✅ 自定义 User-Agent

## 🚀 快速开始

### 安装

添加依赖到你的 `Cargo.toml`：

```toml
[dependencies]
openai4rs = "0.1.4"
tokio = { version = "1.45.1", features = ["full"] }
futures = "0.3.31"
```

或使用 cargo 命令：

```bash
cargo add openai4rs
```

### 基础使用

```rust
use openai4rs::{OpenAI, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let messages = vec![user!("Hello, world!")];
    
    let response = client
        .chat()
        .create(chat_request("gpt-3.5-turbo", &messages))
        .await
        .unwrap();
        
    println!("{:#?}", response);
}
```

## 📚 详细使用指南

### **🗨️ Chat 聊天**

#### 非流式聊天

最简单的聊天方式，一次性获取完整响应：

```rust
use openai4rs::{OpenAI, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let messages = vec![user!("你好，请介绍一下你自己")];
    
    let chat_completion = client
        .chat()
        .create(chat_request("your_model_name", &messages))
        .await
        .unwrap();
        
    println!("{:#?}", chat_completion);
}
```

#### 流式聊天

实时接收响应内容，适合需要逐步显示的场景：

```rust
use futures::StreamExt;
use openai4rs::{OpenAI, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let messages = vec![user!("请写一个关于人工智能的故事")];
    
    let mut stream = client
        .chat()
        .create_stream(chat_request("your_model_name", &messages))
        .await
        .unwrap();
        
    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        // 处理每个响应块
        for choice in chunk.choices.iter() {
            if let Some(content) = &choice.delta.content {
                print!("{}", content);
            }
        }
    }
}
```

#### 🔧 工具调用

让模型能够调用外部工具来增强功能：

```rust
use futures::StreamExt;
use openai4rs::{ChatCompletionToolParam, OpenAI, chat_request, user, ToolChoice};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    
    // 定义工具
    let tools = vec![ChatCompletionToolParam::function(
        "get_current_time",
        "获取当前时间",
        serde_json::json!({
            "type": "object",
            "properties": {},
            "description": "获取当前的日期和时间"
        }),
    )];

    let messages = vec![user!("现在几点了？")];
    
    let mut stream = client
        .chat()
        .create_stream(
            chat_request("your_model_name", &messages)
                .tools(tools)
                .tool_choice(ToolChoice::Auto)
        )
        .await
        .unwrap();
        
    while let Some(result) = stream.next().await {
        match result {
            Ok(chunk) => {
                println!("收到响应: {:#?}", chunk);
            }
            Err(err) => {
                eprintln!("错误: {:#?}", err);
            }
        }
    }
}
```

#### 🧠 思考模式

供应商返回字段为reasoning或reasoning_content都会映射到reasoning字段。
适用于支持思考功能的模型（如 qwen 的 qwq-32b）：

```rust
use futures::StreamExt;
use openai4rs::{OpenAI, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let messages = vec![user!("请解决这个数学问题：如果一个三角形的两边分别是3和4，第三边是5，这是什么类型的三角形？")];
    
    let mut stream = client
        .chat()
        .create_stream(chat_request("qwq-32b", &messages))
        .await
        .unwrap();
        
    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        for choice in chunk.choices.iter() {
            // 显示模型的思考过程
            if choice.delta.is_reasoning() {
                println!("🤔 思考过程:\n{}", choice.delta.get_reasoning_str());
            }
            // 显示最终回答
            if let Some(content) = &choice.delta.content {
                if !content.is_empty() {
                    println!("💡 回答:\n{}", content);
                }
            }
        }
    }
}
```

### 🔗 响应合并与消息映射

#### 合并流式响应输出(使用重载的 `+` 运行符)

将流式响应合并为完整的回复内容：

```rust
use futures::stream::StreamExt;
use openai4rs::{OpenAI, StreamChoice, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let messages = vec![user!("请详细介绍一下 Rust 的所有权机制")];

    let mut stream = client
        .chat()
        .create_stream(chat_request("your_model_name", &messages))
        .await
        .unwrap();

    let mut merged_choice: Option<StreamChoice> = None;
    while let Some(result) = stream.next().await {
        let chat_completion_chunk = result.unwrap();
        let choice = chat_completion_chunk.choices[0].clone();
        merged_choice = Some(match merged_choice {
            Some(l) => l + choice,
            None => choice,
        })
    }
    println!("{:#?}", merged_choice.unwrap());
}
```

#### 将响应映射到消息链

```rust
use futures::stream::StreamExt;
use openai4rs::{OpenAI, StreamChoice, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let mut messages = vec![user!("请详细介绍一下 Rust 的所有权机制")];

    let mut stream = client
        .chat()
        .create_stream(chat_request("your_model_name", &messages))
        .await
        .unwrap();

    let mut merged_choice: Option<StreamChoice> = None;
    while let Some(result) = stream.next().await {
        let chat_completion_chunk = result.unwrap();
        let choice = chat_completion_chunk.choices[0].clone();
        merged_choice = Some(match merged_choice {
            Some(l) => l + choice,
            None => choice,
        })
    }
    messages.push(merged_choice.unwrap().delta.into());

    messages.push(user!("好的, 谢谢你"));

    let chat_completion = client
        .chat()
        .create(chat_request("your_model_name", &messages))
        .await
        .unwrap();

    messages.push(chat_completion.choices[0].message.clone().into())
}
```

### **📝 Completions 文本补全**

#### 非流式补全

```rust
use openai4rs::{OpenAI, completions_request};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    
    let completion = client
        .completions()
        .create(completions_request("your_model_name", "请补全这句话：人工智能的未来"))
        .await
        .unwrap();
        
    println!("补全结果: {:#?}", completion);
}
```

#### 流式补全

```rust
use futures::StreamExt;
use openai4rs::{OpenAI, completions_request};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    
    let mut stream = client
        .completions()
        .create_stream(completions_request("your_model_name", "编写一个快速排序算法："))
        .await
        .unwrap();
        
    while let Some(result) = stream.next().await {
        match result {
            Ok(completion) => {
                println!("补全内容: {:#?}", completion);
            }
            Err(err) => {
                eprintln!("错误: {}", err);
            }
        }
    }
}
```

### **🤖 Models 模型管理**

#### 获取所有可用模型

```rust
use openai4rs::{OpenAI, models_request};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    
    let models = client
        .models()
        .list(models_request())
        .await
        .unwrap();
        
    println!("可用模型:");
    for model in models.data.iter() {
        println!("- {}: {}", model.id, model.created);
    }
}
```

## 🔧 配置选项

### 客户端配置

```rust
use openai4rs::{OpenAI, Config};

// 基础配置
let client = OpenAI::new("your_api_key", "https://api.openai.com/v1");

// 使用环境变量创建客户端
// 环境变量: OPENAI_API_KEY, OPENAI_BASE_URL, OPENAI_TIMEOUT, OPENAI_CONNECT_TIMEOUT, OPENAI_RETRY_COUNT, OPENAI_PROXY, OPENAI_USER_AGENT
let client = OpenAI::from_env().unwrap();

// 使用自定义配置创建客户端
let mut config = Config::new("your_api_key".to_string(), "https://api.openai.com/v1".to_string());
config.set_retry_count(3)                                   // 设置最大重试次数为3
      .set_timeout_seconds(120)                             // 设置请求超时为120秒
      .set_connect_timeout_seconds(5)                       // 设置连接超时为5秒
      .set_proxy(Some("http://localhost:8080".to_string())) // 设置HTTP代理
      .set_user_agent(Some("MyApp/1.0".to_string()));       // 设置自定义User-Agent

let client = OpenAI::with_config(config);

// 动态更新客户端配置
client.update_config(|config| {
    config.set_timeout_seconds(180)
          .set_retry_count(2);
}).await;

// 异步获取和设置 API 配置
#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "https://api.openai.com/v1");
    
    // 获取配置信息
    let base_url = client.get_base_url().await;
    let api_key = client.get_api_key().await;
    
    // 更新配置信息
    client.set_base_url("https://api.custom-provider.com/v1".to_string()).await;
    client.set_api_key("new-api-key".to_string()).await;
}
```

### 请求参数配置

```rust
use openai4rs::{chat_request, user};

let messages = vec![user!("Hello")];

let request = chat_request("gpt-3.5-turbo", &messages)
    .temperature(0.7)             // 控制随机性
    .max_completion_tokens(1000)  // 最大token数
    .top_p(0.9)                   // 核心采样
    .frequency_penalty(0.1)       // 频率惩罚
    .presence_penalty(0.1);       // 存在惩罚
```

## 📖 更多示例

查看 [examples](examples/) 目录获取更多使用示例：

- [基础聊天](examples/chat.rs)
- [流式响应](examples/chat_stream.rs)
- [工具调用](examples/tool.rs)
- [思考模式](examples/chat_reasoning_stream.rs)
- [HTTP请求配置](examples/http_config.rs)
- [自定义配置](examples/custom_config.rs)

## 📄 许可证

本项目采用 [Apache-2.0 许可证](LICENSE)。

## 🔗 相关链接

- [文档](https://docs.rs/openai4rs)
- [Crates.io](https://crates.io/crates/openai4rs)
- [GitHub 仓库](https://github.com/zhangzhenxiang666/openai4rs)
- [问题反馈](https://github.com/zhangzhenxiang666/openai4rs/issues)
