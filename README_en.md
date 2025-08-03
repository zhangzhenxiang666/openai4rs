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
- ✅ Reasoning mode

### 📝 Completions

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

## 🚀 Quick Start

### Installation

Add the dependencies to your `Cargo.toml`:

```toml
[dependencies]
openai4rs = "0.1.3"
tokio = { version = "1.45.1", features = ["full"] }
futures = "0.3.31"
```

Or use the cargo command:

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

The simplest way to chat, getting the complete response at once:

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

Receive response content in real-time, suitable for scenarios that require progressive display:

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

Allow the model to call external tools to enhance its functionality:

```rust
use futures::StreamExt;
use openai4rs::{ChatCompletionToolParam, OpenAI, chat_request, user, ToolChoice};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    
    // Define the tool
    let tools = vec![ChatCompletionToolParam::function(
        "get_current_time",
        "Get the current time",
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
                println!("Received chunk: {:#?}", chunk);
            }
            Err(err) => {
                eprintln!("Error: {:#?}", err);
            }
        }
    }
}
```

#### 🧠 Reasoning Mode

Fields returned by the provider as `reasoning` or `reasoning_content` will be mapped to the `reasoning` field. Applicable to models that support reasoning functionality (e.g., qwen's qwq-32b):

```rust
use futures::StreamExt;
use openai4rs::{OpenAI, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let messages = vec![user!("Please solve this math problem: If two sides of a triangle are 3 and 4, and the third side is 5, what type of triangle is it?")];
    
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
                println!("🤔 Reasoning Process:\n{}", choice.delta.get_reasoning_str());
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

Using the `Apply` trait makes it more convenient to process stream data:

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
        println!("Processing chunk: {:#?}", chunk);
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
            // You can perform asynchronous operations here
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            println!("Async processing: {:#?}", chunk);
        })
        .await;
}
```

##### Asynchronous Processing with State Capture

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

    // Collect the complete AI output
    let complete_response = stream
        .apply_with_capture_async(String::new(), |mut accumulated, result| {
            Box::pin(async move {
                let chunk = result.expect("Error processing stream");
                for choice in chunk.choices.iter() {
                    if let Some(content) = choice.delta.content.as_ref() {
                        print!("{}", content); // Display in real-time
                        accumulated.push_str(content); // Accumulate content
                    }
                }
                accumulated
            })
        })
        .await;

    println!("\n\nComplete Response:\n{}", complete_response);
}
```

### 🔗 Response Merging and Message Mapping

#### Merge Streaming Response Output (using the overloaded `+` operator)

Merge the streaming response into a complete reply:

```rust
use futures::stream::StreamExt;
use openai4rs::{OpenAI, StreamChoice, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let messages = vec![user!("Please explain Rust's ownership system in detail")];

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

#### Map Response to Message Chain

```rust
use futures::stream::StreamExt;
use openai4rs::{OpenAI, StreamChoice, chat_request, user};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    let mut messages = vec![user!("Please explain Rust's ownership system in detail")];

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

    messages.push(user!("Okay, thank you"));

    let chat_completion = client
        .chat()
        .create(chat_request("your_model_name", &messages))
        .await
        .unwrap();

    messages.push(chat_completion.choices[0].message.clone().into())
}
```

### **📝 Completions**

#### Non-streaming Completion

```rust
use openai4rs::{OpenAI, completions_request};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    
    let completion = client
        .completions()
        .create(completions_request("your_model_name", "Complete this sentence: The future of artificial intelligence is"))
        .await
        .unwrap();
        
    println!("Completion result: {:#?}", completion);
}
```

#### Streaming Completion

```rust
use futures::StreamExt;
use openai4rs::{OpenAI, completions_request};

#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "your_base_url");
    
    let mut stream = client
        .completions()
        .create_stream(completions_request("your_model_name", "Write a quicksort algorithm:"))
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

### **🤖 Models Management**

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
use openai4rs::{OpenAI, Config};

// Basic configuration
let client = OpenAI::new("your_api_key", "https://api.openai.com/v1");

// Create client from environment variables
// Environment variables: OPENAI_API_KEY, OPENAI_BASE_URL, OPENAI_TIMEOUT, OPENAI_CONNECT_TIMEOUT, OPENAI_RETRY_COUNT, OPENAI_PROXY, OPENAI_USER_AGENT
let client = OpenAI::from_env().unwrap();

// Create client with custom configuration
let mut config = Config::new("your_api_key".to_string(), "https://api.openai.com/v1".to_string());
config.set_retry_count(3)                                   // Set max retry attempts to 3
      .set_timeout_seconds(120)                             // Set request timeout to 120 seconds
      .set_connect_timeout_seconds(5)                       // Set connection timeout to 5 seconds
      .set_proxy(Some("http://localhost:8080".to_string())) // Set HTTP proxy
      .set_user_agent(Some("MyApp/1.0".to_string()));       // Set custom User-Agent

let client = OpenAI::with_config(config);

// Dynamically update client configuration
client.update_config(|config| {
    config.set_timeout_seconds(180)
          .set_retry_count(2);
}).await;

// Async API configuration access and updates
#[tokio::main]
async fn main() {
    let client = OpenAI::new("your_api_key", "https://api.openai.com/v1");
    
    // Get configuration information
    let base_url = client.get_base_url().await;
    let api_key = client.get_api_key().await;
    
    // Update configuration information
    client.set_base_url("https://api.custom-provider.com/v1".to_string()).await;
    client.set_api_key("new-api-key".to_string()).await;
}
```

### Request Parameter Configuration

```rust
use openai4rs::{chat_request, user};

let messages = vec![user!("Hello")];

let request = chat_request("gpt-3.5-turbo", &messages)
    .temperature(0.7)          // Controls randomness
    .max_completion_tokens(1000)   // Maximum number of tokens
    .top_p(0.9)                  // Nucleus sampling
    .frequency_penalty(0.1)      // Frequency penalty
    .presence_penalty(0.1);      // Presence penalty
```

## 📖 More Examples

Check the [examples](examples/) directory for more usage examples:

- [Basic Chat](examples/chat.rs)
- [Streaming Response](examples/chat_stream.rs)
- [Tool Calling](examples/tool.rs)
- [Reasoning Mode](examples/chat_reasoning_stream.rs)
- [HTTP Request Configuration](examples/http_config.rs)
- [Custom Configuration](examples/custom_config.rs)

## 📄 License

This project is licensed under the [Apache-2.0 License](https://www.google.com/search?q=LICENSE).

## 🔗 Related Links

- [Documentation](https://docs.rs/openai4rs)
- [Crates.io](https://crates.io/crates/openai4rs)
- [GitHub Repository](https://github.com/zhangzhenxiang666/openai4rs)
- [Issue Tracker](https://github.com/zhangzhenxiang666/openai4rs/issues)
