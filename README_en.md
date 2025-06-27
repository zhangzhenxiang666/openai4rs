# openai4rs

[![Crates.io](https://img.shields.io/crates/v/openai4rs)](https://crates.io/crates/openai4rs)
[![Documentation](https://docs.rs/openai4rs/badge.svg)](https://docs.rs/openai4rs)
[![License](https://img.shields.io/crates/l/openai4rs)](LICENSE)

English | [简体中文](README.md)

An asynchronous Rust crate based on `tokio` and `reqwest` for interacting with large model providers that follow the OpenAI specification.

## ✨ Features

### 🗨️ Chat

- ✅ Streaming responses
- ✅ Tool calling
- ✅ Reasoning mode

### 📝 Completions

- ✅ Non-streaming responses
- ✅ Streaming responses

### 🤖 Models

- ✅ Get model list
- ✅ Get single model information

## 🚀 Quick Start

### Installation

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
openai4rs = "0.1.1"
tokio = { version = "1.45.1", features = ["full"] }
futures = "0.3.31"
```

Or use cargo command:

```bash
cargo add openai4rs
```

### Basic Usage

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

## 📚 Detailed Usage Guide

### **🗨️ Chat**

#### Non-streaming Chat

The simplest way to chat, getting complete responses at once:

```rust
use openai4rs::{OpenAI, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let messages = vec![user!("Hello, please introduce yourself")];
    
    let chat_completion = client
        .chat()
        .create(chat_request("your_model_name", &messages))
        .await
        .unwrap();
        
    println!("{:#?}", chat_completion);
}
```

#### Streaming Chat

Receive response content in real-time, suitable for scenarios requiring progressive display:

```rust
use futures::StreamExt;
use openai4rs::{OpenAI, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let messages = vec![user!("Please write a story about artificial intelligence")];
    
    let mut stream = client
        .chat()
        .create_stream(chat_request("your_model_name", &messages))
        .await
        .unwrap();
        
    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        // Process each response chunk
        for choice in chunk.choices.iter() {
            if let Some(content) = &choice.delta.content {
                print!("{}", content);
            }
        }
    }
}
```

#### 🔧 Tool Calling

Enable models to call external tools to enhance functionality:

```rust
use futures::StreamExt;
use openai4rs::{ChatCompletionToolParam, OpenAI, chat_request, user, ToolChoice};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    
    // Define tools
    let tools = vec![ChatCompletionToolParam::function(
        "get_current_time",
        "Get current time",
        serde_json::json!({
            "type": "object",
            "properties": {},
            "description": "Get the current date and time"
        }),
    )];

    let messages = vec![user!("What time is it now?")];
    
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
                println!("Received response: {:#?}", chunk);
            }
            Err(err) => {
                eprintln!("Error: {:#?}", err);
            }
        }
    }
}
```

#### 🧠 Reasoning Mode

Suitable for models that support reasoning functionality (such as qwen's qwq-32b):

```rust
use futures::StreamExt;
use openai4rs::{OpenAI, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let messages = vec![user!("Please solve this math problem: If a triangle has two sides of 3 and 4, and the third side is 5, what type of triangle is this?")];
    
    let mut stream = client
        .chat()
        .create_stream(chat_request("qwq-32b", &messages))
        .await
        .unwrap();
        
    while let Some(result) = stream.next().await {
        let chunk = result.unwrap();
        for choice in chunk.choices.iter() {
            // Display the model's reasoning process
            if choice.delta.is_reasoning() {
                println!("🤔 Reasoning process:\n{}", choice.delta.get_reasoning_str());
            }
            // Display the final answer
            if let Some(content) = &choice.delta.content {
                if !content.is_empty() {
                    println!("💡 Answer:\n{}", content);
                }
            }
        }
    }
}
```

### 🔄 Stream Processing Tools

#### Apply - Synchronous Iteration

Use the `Apply` trait to handle stream data more conveniently:

```rust
use openai4rs::{Apply, OpenAI, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let messages = vec![user!("Please introduce the Rust programming language")];

    let stream = client
        .chat()
        .create_stream(chat_request("your_model_name", &messages))
        .await
        .unwrap();

    // Synchronously process each response chunk
    stream.apply(|result| {
        let chunk = result.unwrap();
        println!("Processing response chunk: {:#?}", chunk);
    });
}
```

#### Apply - Asynchronous Iteration

##### Simple Asynchronous Processing

```rust
use openai4rs::{Apply, OpenAI, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let messages = vec![user!("Explain what machine learning is")];
    
    let stream = client
        .chat()
        .create_stream(chat_request("your_model_name", &messages))
        .await
        .unwrap();
        
    stream
        .apply_async(|result| async move {
            let chunk = result.unwrap();
            // Can perform asynchronous operations here
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            println!("Async processing: {:#?}", chunk);
        })
        .await;
}
```

##### Asynchronous Processing with External State Capture

```rust
use openai4rs::{Apply, OpenAI, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let messages = vec![user!("Please write a poem about programming")];

    let stream = client
        .chat()
        .create_stream(chat_request("your_model_name", &messages))
        .await
        .unwrap();

    // Collect complete AI output
    let complete_response = stream
        .apply_with_capture_async(String::new(), |accumulated, result| {
            Box::pin(async move {
                let chunk = result.expect("Error processing stream");
                for choice in chunk.choices.iter() {
                    if let Some(content) = choice.delta.content.as_ref() {
                        print!("{}", content); // Real-time display
                        accumulated.push_str(content); // Accumulate content
                    }
                }
            })
        })
        .await;

    println!("\n\nComplete response:\n{}", complete_response);
}
```

### **📝 Completions**

#### Non-streaming Completion

```rust
use openai4rs::{OpenAI, comletions_request};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    
    let completion = client
        .completions()
        .create(comletions_request("your_model_name", "Please complete this sentence: The future of artificial intelligence"))
        .await
        .unwrap();
        
    println!("Completion result: {:#?}", completion);
}
```

#### Streaming Completion

```rust
use futures::StreamExt;
use openai4rs::{OpenAI, comletions_request};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    
    let mut stream = client
        .completions()
        .create_stream(comletions_request("your_model_name", "Write a quicksort algorithm:"))
        .await
        .unwrap();
        
    while let Some(result) = stream.next().await {
        match result {
            Ok(completion) => {
                println!("Completion content: {:#?}", completion);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }
}
```

### **🤖 Models**

#### Get All Available Models

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
        
    println!("Available models:");
    for model in models.data.iter() {
        println!("- {}: {}", model.id, model.created);
    }
}
```

## 🔧 Configuration Options

### Client Configuration

```rust
use openai4rs::{OpenAI};

// Basic configuration
let client = OpenAI::new("your_api_key", "https://api.openai.com/v1");

```

### Request Parameter Configuration

```rust
use openai4rs::{chat_request, user};

let messages = vec![user!("Hello")];

let request = chat_request("gpt-3.5-turbo", &messages)
    .temperature(0.7)             // Control randomness
    .max_completion_tokens(1000)  // Maximum token count
    .top_p(0.9)                   // Nucleus sampling
    .frequency_penalty(0.1)       // Frequency penalty
    .presence_penalty(0.1);       // Presence penalty
```

## 📖 More Examples

Check the [examples](examples/) directory for more usage examples:

- [Basic Chat](examples/chat.rs)
- [Streaming Response](examples/chat_stream.rs)
- [Tool Calling](examples/tool.rs)
- [Reasoning Mode](examples/chat_reasoning_stream.rs)

## 📄 License

This project is licensed under the [MIT License](LICENSE).

## 🔗 Related Links

- [Documentation](https://docs.rs/openai4rs)
- [Crates.io](https://crates.io/crates/openai4rs)
- [GitHub Repository](https://github.com/zhangzhenxiang666/openai4rs)
- [Issue Tracker](https://github.com/zhangzhenxiang666/openai4rs/issues)
