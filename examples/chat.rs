use dotenvy::dotenv;
use openai4rs::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let client = OpenAI::from_env().unwrap();

    let model = "your model name";

    let messages = vec![user!("Introduce the Rust programming language.")];
    let res = client
        .chat()
        .create(chat_request(model, &messages))
        .await
        .unwrap();
    println!("{:#?}", res);
}
