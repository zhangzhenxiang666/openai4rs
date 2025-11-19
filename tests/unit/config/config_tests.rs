use std::time::Duration;

use http::HeaderValue;
use openai4rs::Config;

#[test]
fn test_config_builder() {
    let config = Config::builder()
        .api_key("test-key".to_string())
        .base_url("https://api.test.com/v1".to_string())
        .retry_count(3)
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(15))
        .proxy("http://proxy.test.com:8080")
        .user_agent(HeaderValue::from_str("TestAgent/1.0").unwrap())
        .build()
        .unwrap();

    assert_eq!(config.api_key(), "test-key");
    assert_eq!(config.base_url(), "https://api.test.com/v1");
    assert_eq!(config.retry_count(), 3);
    assert_eq!(config.timeout(), Duration::from_secs(120));
    assert_eq!(config.connect_timeout(), Duration::from_secs(15));
    assert_eq!(
        config.proxy().map(|s| s.as_str()),
        Some("http://proxy.test.com:8080")
    );
    assert_eq!(
        config.user_agent(),
        Some(&HeaderValue::from_str("TestAgent/1.0").unwrap())
    );
}

#[test]
fn test_config_builder_defaults() {
    let config = Config::builder()
        .api_key("test-key".to_string())
        .base_url("https://api.test.com/v1".to_string())
        .build()
        .unwrap();

    assert_eq!(config.retry_count(), 5); // default value
    assert_eq!(config.timeout(), Duration::from_secs(300)); // default value
    assert_eq!(config.connect_timeout(), Duration::from_secs(10)); // default value
    assert_eq!(config.proxy(), None); // default value
    assert_eq!(config.user_agent(), None); // default value
}

#[test]
fn test_config_new() {
    let config = Config::new(
        "test-key".to_string(),
        "https://api.test.com/v1".to_string(),
    );

    assert_eq!(config.api_key(), "test-key");
    assert_eq!(config.base_url(), "https://api.test.com/v1");
}

#[test]
fn test_config_setters() {
    let mut config = Config::new("old-key".to_string(), "https://old-api.com/v1".to_string());

    config
        .with_api_key("new-key")
        .with_base_url("https://new-api.com/v1")
        .with_retry_count(2)
        .with_timeout(Duration::from_secs(30))
        .with_connect_timeout(Duration::from_secs(5))
        .with_proxy("http://proxy.example.com:8080")
        .with_user_agent(HeaderValue::from_str("CustomAgent/2.0").unwrap());

    assert_eq!(config.api_key(), "new-key");
    assert_eq!(config.base_url(), "https://new-api.com/v1");
    assert_eq!(config.retry_count(), 2);
    assert_eq!(config.timeout(), Duration::from_secs(30));
    assert_eq!(config.connect_timeout(), Duration::from_secs(5));
    assert_eq!(
        config.proxy().map(|s| s.as_str()),
        Some("http://proxy.example.com:8080")
    );
    assert_eq!(
        config.user_agent(),
        Some(&HeaderValue::from_str("CustomAgent/2.0").unwrap())
    );
}

#[tokio::test]
async fn test_build_openai() {
    let client = Config::builder()
        .api_key("test-key".to_string())
        .base_url("https://api.test.com/v1".to_string())
        .build_openai()
        .unwrap();

    assert_eq!(client.api_key().await, "test-key");
    assert_eq!(client.base_url().await, "https://api.test.com/v1");
}
