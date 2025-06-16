use crate::chat::Chat;
use reqwest::Client;
use std::{
    cell::OnceCell,
    sync::{Arc, RwLock},
};

pub(crate) struct Config {
    api_key: String,
    base_url: String,
}

impl Config {
    pub fn get_api_key(&self) -> String {
        self.api_key.to_string()
    }
    pub fn get_base_url(&self) -> String {
        self.base_url.to_string()
    }
}

pub struct OpenAI {
    config: Arc<RwLock<Config>>,
    chat: OnceCell<Chat>,
    client: Arc<Client>,
}

impl OpenAI {
    pub fn new(api_key: &str, base_url: &str) -> Self {
        let client = Arc::new(Client::new());
        let config = Arc::new(RwLock::new(Config {
            api_key: api_key.into(),
            base_url: base_url.into(),
        }));
        Self {
            chat: OnceCell::new(),
            client,
            config,
        }
    }

    pub fn from_env() -> Result<Self, String> {
        let api_key = std::env::var("OPENAI_API_KEY").map_err(|_| "OPENAI_API_KEY not set")?;
        let base_url =
            std::env::var("OPENAI_BASE_URL").unwrap_or("https://api.openai.com/v1".to_string());
        Ok(Self::new(&api_key, &base_url))
    }
}

impl OpenAI {
    pub fn chat(&self) -> &Chat {
        self.chat
            .get_or_init(|| Chat::new(Arc::clone(&self.config), Arc::clone(&self.client)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chat::*, error::OpenAIError, user};
    use dotenvy::dotenv;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_chat() {
        dotenv().ok();
        let client = OpenAI::from_env().unwrap();
        let messages = vec![user!("Hello")];
        let res = client
            .chat()
            .create(
                chat_request("meta-llama/llama-3.3-8b-instruct:free", &messages).temperature(0.0),
            )
            .await
            .unwrap();
        assert_eq!(
            Some("Hello! How can I assist you today?".into()),
            res.choices[0].message.content
        );
    }

    #[tokio::test]
    async fn test_openai_error_authentication() {
        let base_url = "https://openrouter.ai/api/v1";
        let api_key = "******";
        let client = OpenAI::new(api_key, base_url);
        let messages = vec![user!("Hello")];
        let mut stream = client
            .chat()
            .create_stream(
                chat_request("meta-llama/llama-3.3-8b-instruct:free", &messages).temperature(0.0),
            )
            .await
            .expect("Request failed");
        let mut flag = true;
        while let Some(result) = stream.next().await {
            flag = false;
            match result {
                Ok(_) => panic!("Unexpected success response"),
                Err(err) => match err {
                    OpenAIError::Authentication(_) => {
                        break;
                    }
                    _ => {
                        panic!("Unexpected error")
                    }
                },
            }
        }
        if flag {
            panic!("No response received")
        }
    }
}
