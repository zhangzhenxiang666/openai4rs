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
- ✅ 多轮对话
- ✅ 视觉（Vision）API（如果模型支持）

### 📝 Completions 文本补全 (Legacy)

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
- ✅ 全局请求头
- ✅ 全局查询参数
- ✅ 全局请求体

### 🎯 拦截器

- ✅ 全局拦截器
- ✅ 模块拦截器

## 🚀 快速开始

### 安装

添加依赖到你的 `Cargo.toml`：

```toml
[dependencies]
openai4rs = "0.1.7"
tokio = { version = "1.45.1", features = ["full"] }
futures = "0.3.31"
dotenvy = "0.15"
```

或使用 cargo 命令：

```bash
cargo add openai4rs
```

### 基础使用

```rust
use dotenvy::dotenv;
use openai4rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let model = "Qwen/Qwen3-235B-A22B-Instruct-2507";
    let messages = vec![
        system!("You are a helpful assistant."),
        user!("Introduce the Rust programming language in one sentence."),
    ];

    let request = chat_request(model, &messages);

    println!("Sending request to model: {}...", model);

    let response = client.chat().create(request).await?;

    if let Some(content) = response.content() {
        println!("\nResponse:\n{}", content);
    } else {
        println!("\nNo content in response.");
    }

    Ok(())
}

```

## 📚 核心用法

### **🗨️ Chat 聊天**

#### 流式聊天

实时接收响应内容，适合需要逐步显示的场景：

```rust
use std::io::{self, Write};

use dotenvy::dotenv;
use futures::StreamExt;
use openai4rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let model = "Qwen/Qwen3-235B-A22B-Instruct-2507";
    let messages = vec![
        system!(content: "You are a helpful assistant."),
        user!(content: "Introduce the Rust programming language in one sentence."),
    ];

    let request = chat_request(model, &messages).build()?;

    println!("Sending request to model: {}...", model);
    
    let mut stream = client.chat().create_stream(request).await?;
    let mut first_content = true;

    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                if chunk.has_content() {
                    if first_content {
                        println!("\n========Response========");
                        first_content = false;
                    }
                    if let Some(content) = chunk.content() {
                        print!("{}", content);
                        io::stdout().flush()?;
                    }
                }
            }
            Err(e) => {
                eprintln!("\nAn error occurred during streaming: {}", e);
                break;
            }
        }
    }
    println!();

    Ok(())
}
```

#### 🔧 工具调用

让模型能够调用外部工具来增强功能：

```rust
use dotenvy::dotenv;
use openai4rs::*;

// Mock function to get weather data
fn get_current_weather(location: &str, unit: Option<&str>) -> String {
    // In a real application, this would call an external weather API.
    let unit = unit.unwrap_or("celsius");
    format!(
        "The current weather in {} is 22 degrees {}.",
        location, unit
    )
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let model = "Qwen/Qwen3-235B-A22B-Instruct-2507";

    // 1. Define the tool (function)
    let weather_tool_params = Parameters::object()
        .property(
            "location",
            Parameters::string()
                .description("The city and state, e.g. San Francisco, CA")
                .build(),
        )
        .property(
            "unit",
            Parameters::string()
                .description("The unit of temperature, e.g. celsius or fahrenheit")
                .build(),
        )
        .require("location")
        .build()?;

    let weather_tool = ChatCompletionToolParam::function(
        "get_current_weather",
        "Get the current weather in a given location",
        weather_tool_params,
    );

    // 2. Create the initial message and request
    let messages = vec![
        system!(content = "You are a helpful assistant."),
        user!(content = "What's the weather like in Boston today?"),
    ];

    let request = chat_request(model, &messages)
        .tools(vec![weather_tool])
        .tool_choice(ToolChoice::Auto)
        .build()?;

    println!("Sending request to model: {}...", model);

    let response = client.chat().create(request).await?;
    println!("Initial response: {:#?}", response);

    // 3. Check if the model wants to call a tool
    if response.has_tool_calls() {
        println!("\nModel wants to call a tool.");
        let tool_calls = response.tool_calls().unwrap();

        // For simplicity, we'll only handle the first tool call
        if let Some(tool_call) = tool_calls.first() {
            let function_name = &tool_call.function.name;
            let arguments_str = &tool_call.function.arguments;

            if function_name == "get_current_weather" {
                let args: serde_json::Value = serde_json::from_str(arguments_str)?;
                let location = args["location"].as_str().unwrap_or("Unknown");
                let unit = args["unit"].as_str();

                println!(
                    "Calling function '{}' with arguments: location='{}', unit='{:?}'",
                    function_name, location, unit
                );

                // 4. Call the function and get the result
                let function_result = get_current_weather(location, unit);
                println!("Function result: {}", function_result);

                // 5. Send the function result back to the model
                let mut new_messages = messages.clone();
                new_messages.push(response.first_choice_message().unwrap().clone().into());
                new_messages.push(tool!(
                    tool_call_id = tool_call.function.id.clone(),
                    content = function_result
                ));

                let follow_up_request = chat_request(model, &new_messages).build()?;

                let final_response = client.chat().create(follow_up_request).await?;
                if let Some(content) = final_response.content() {
                    println!("\nFinal Assistant Response:\n{}", content);
                }
            }
        }
    } else {
        // If no tool call, just print the content
        if let Some(content) = response.content() {
            println!("\nAssistant Response:\n{}", content);
        }
    }

    Ok(())
}
```

#### 🧠 多轮对话

维护一个具有上下文的多轮对话：

```rust
use dotenvy::dotenv;
use openai4rs::*;
use std::io::{stdin, stdout, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let model = "Qwen/Qwen3-235B-A22B-Instruct-2507";
    let mut messages = vec![system!(content: "You are a helpful assistant.")];

    loop {
        print!("You: ");
        stdout().flush()?;
        let mut user_input = String::new();
        stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        if user_input.eq_ignore_ascii_case("exit") {
            println!("Goodbye!");
            break;
        }

        messages.push(user!(content: user_input));

        let request = chat_request(model, &messages);

        let response = client.chat().create(request).await?;
        if let Some(content) = response.content() {
            println!("Assistant: {}\n", content);
            messages.push(assistant!(content));
        } else {
            println!("Assistant: No response.\n");
        }
    }

    Ok(())
}
```

### **🔧 高级配置**

#### 客户端配置

```rust
use dotenvy::dotenv;
use openai4rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    // Get the API key from the environment
    let api_key = std::env::var("OPENAI_API_KEY")?;
    let base_url = std::env::var("OPENAI_BASE_URL")?;
    // 1. Basic client with default settings
    let basic_client = OpenAI::new(&api_key, &base_url);

    // 2. Client with a custom base URL (e.g., for a proxy or a different provider)
    let _custom_base_url_client = Config::builder()
        .api_key(&api_key)
        .base_url(&base_url) // Replace with your custom base URL
        .build_openai()?;

    // 3. Client with a proxy
    let proxy_config = Config::builder()
        .api_key(&api_key)
        .base_url(&base_url)
        .proxy("http://proxy.example.com:8080")
        .build()?;
    let _proxy_client = OpenAI::with_config(proxy_config);

    // 4. Client with custom timeout
    let timeout_config = Config::builder()
        .api_key(&api_key)
        .base_url(&base_url)
        .timeout_seconds(120)
        .build()?;
    let _timeout_client = OpenAI::with_config(timeout_config);

    // For demonstration, we'll use the basic client to make a simple request.
    // In a real application, you would use the client that best fits your needs.

    let model = "Qwen/Qwen3-235B-A22B-Instruct-2507";
    let messages = vec![user!(content: "Ping to check if the client is working.")];
    let request = chat_request(model, &messages);

    println!("Testing basic client...");
    match basic_client.chat().create(request).await {
        Ok(response) => {
            if let Some(content) = response.content() {
                println!("Success: {}", content);
            }
        }
        Err(e) => {
            eprintln!("Error with basic client: {}", e);
        }
    }

    Ok(())
}
```

## 📖 运行示例

查看 [examples](examples/) 目录获取更多使用示例：

- [01. 基础聊天](examples/01_simple_chat.rs)
- [02. 流式响应](examples/02_streaming_chat.rs)
- [03. 多轮对话](examples/03_multi_turn_chat.rs)
- [04. 工具调用](examples/04_tool_use.rs)
- [05. 客户端配置](examples/05_client_configuration.rs)
- [06. 视觉（Vision）API](examples/06_vision.rs) (如果模型支持)
- [07. 思维模型（Thinking Model）](examples/07_thinking_model.rs) (如果模型支持复杂推理)
- [08. 全局拦截器](examples/08_interceptor_example.rs)
- [09. 模块拦截器](examples/09_module_interceptor_example.rs)

你可以通过以下命令运行示例：

```bash
# 设置环境变量
export OPENAI_API_KEY=your_api_key
export OPENAI_BASE_URL=your_base_url # 可选, 默认为 https://api.openai.com/v1

# 运行示例
cargo run --example 01_simple_chat
cargo run --example 02_streaming_chat
# ... 其他示例
```

## 📄 许可证

本项目采用 [Apache-2.0 许可证](LICENSE)。

## 🔗 相关链接

- [文档](https://docs.rs/openai4rs)
- [Crates.io](https://crates.io/crates/openai4rs)
- [GitHub 仓库](https://github.com/zhangzhenxiang666/openai4rs)
- [问题反馈](https://github.com/zhangzhenxiang666/openai4rs/issues)
