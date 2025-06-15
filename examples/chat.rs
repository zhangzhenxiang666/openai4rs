use dotenvy::dotenv;
use openai4rs::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();
    let messages = vec![user!("Introduce the Rust programming language.")];
    let res = client
        .chat()
        .create(chat_request(
            "meta-llama/llama-3.3-8b-instruct:free",
            &messages,
        ))
        .await
        .unwrap();
    println!("{:#?}", res);
}
