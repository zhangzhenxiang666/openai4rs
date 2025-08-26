# openai4rs

[![Crates.io](https://img.shields.io/crates/v/openai4rs)](https://crates.io/crates/openai4rs)
[![Documentation](https://docs.rs/openai4rs/badge.svg)](https://docs.rs/openai4rs)
[![License](https://img.shields.io/crates/l/openai4rs)](LICENSE)

[简体中文](README.md) | English

An asynchronous Rust crate based on `tokio` and `reqwest` for interacting with large model providers that adhere to the OpenAI specification.

## ✨ Features

### 🗨️ Chat

- ✅ Streaming responses
- ✅ Tool calling
- ✅ Multi-turn conversations
- ✅ Vision API (if supported by the model)

### 📝 Completions (Legacy)

- ✅ Non-streaming responses
- ✅ Streaming responses

### 🤖 Models

- ✅ List models
- ✅ Retrieve a single model

### 🔄 HTTP Request Control

- ✅ Configurable retry attempts
- ✅ Configurable request timeout
- ✅ Configurable connection timeout
- ✅ HTTP proxy support
- ✅ Custom User-Agent
- ✅ Custom Headers

## 🚀 Quick Start

### Installation

Add the dependencies to your `Cargo.toml`:

```toml
[dependencies]
openai4rs = "0.1.5"
tokio = { version = "1.45.1", features = ["full"] }
futures = "0.3.31"
dotenvy = "0.15"
```

Or use the cargo command:

```bash
cargo add openai4rs
```

### Basic Usage

```rust
use dotenvy::dotenv;
use openai4rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let model = "Qwen/Qwen3-Coder-480B-A35B-Instruct";
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

## 📚 Core Usage

### **🗨️ Chat**

#### Streaming Chat

Receive response content in real-time, suitable for scenarios that require progressive display:

```rust
use std::io::{self, Write};

use dotenvy::dotenv;
use futures::StreamExt;
use openai4rs::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let model = "Qwen/Qwen3-Coder-480B-A35B-Instruct";
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

#### 🔧 Tool Calling

Allow the model to call external tools to enhance its functionality:

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

    let model = "Qwen/Qwen3-Coder-480B-A35B-Instruct";

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

#### 🧠 Multi-turn Conversations

Maintain a multi-turn conversation with context:

```rust
use dotenvy::dotenv;
use openai4rs::*;
use std::io::{stdin, stdout, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    let model = "Qwen/Qwen3-Coder-480B-A35B-Instruct";
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

### **🔧 Advanced Configuration**

#### Client Configuration

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
        .api_key(api_key.clone())
        .base_url(base_url.clone()) // Replace with your custom base URL
        .build_openai()?;

    // 3. Client with a proxy
    let proxy_config = Config::builder()
        .api_key(api_key.clone())
        .base_url(base_url.clone())
        .http_config(
            HttpConfig::builder()
                .proxy("http://proxy.example.com:8080".to_string())
                .build()
                .unwrap(),
        )
        .build()?;
    let _proxy_client = OpenAI::with_config(proxy_config);

    // 4. Client with custom timeout
    let timeout_config = Config::builder()
        .api_key(api_key)
        .base_url(base_url.clone())
        .http_config(HttpConfig::builder().timeout_seconds(120).build().unwrap())
        .build()?;
    let _timeout_client = OpenAI::with_config(timeout_config);

    // For demonstration, we'll use the basic client to make a simple request.
    // In a real application, you would use the client that best fits your needs.

    let model = "Qwen/Qwen3-Coder-480B-A35B-Instruct";
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

## 📖 Running Examples

Check the [examples](examples/) directory for more usage examples:

- [01. Basic Chat](examples/01_simple_chat.rs)
- [02. Streaming Response](examples/02_streaming_chat.rs)
- [03. Multi-turn Conversation](examples/03_multi_turn_chat.rs)
- [04. Tool Calling](examples/04_tool_use.rs)
- [05. Client Configuration](examples/05_client_configuration.rs)
- [06. Vision API](examples/06_vision.rs) (if supported by the model)
- [07. Thinking Model](examples/07_thinking_model.rs) (if the model supports complex reasoning)

You can run the examples with the following commands:

```bash
# Set environment variables
export OPENAI_API_KEY=your_api_key
export OPENAI_BASE_URL=your_base_url # Optional, defaults to https://api.openai.com/v1

# Run examples
cargo run --example 01_simple_chat
cargo run --example 02_streaming_chat
# ... other examples
```

## 📄 License

This project is licensed under the [Apache-2.0 License](LICENSE).

## 🔗 Related Links

- [Documentation](https://docs.rs/openai4rs)
- [Crates.io](https://crates.io/crates/openai4rs)
- [GitHub Repository](https://github.com/zhangzhenxiang666/openai4rs)
- [Issue Tracker](https://github.com/zhangzhenxiang666/openai4rs/issues)
