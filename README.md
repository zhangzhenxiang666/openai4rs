# openai4rs

简体中文 | [English](README_en.md)

## 简介

openai4rs是一个非官方实现的基于tokio和reqwest的异步与大模型供应商以openai规范交互的rust实现的crate。

## 使用指南

1. chat聊天(非流式)

```rust
use openai4rs::{OpenAI, user, chat_request}

#[tokio::main]
async fn main(){
    let base_url = "your baseurl";
    let api_key = "your api key";
    let client = OpenAI::new(api_key, base_url);
    let messages = vec![user!("hello")];
    let response = client.chat().create(chat_request("your modl name", &messages)).await.unwrap();
    println!("{:#?}", response);
}
```
