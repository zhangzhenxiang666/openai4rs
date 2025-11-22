use http::HeaderValue;
use openai4rs::Config;
use std::time::Duration;

#[test]
fn test_config_builder() {
    let config = Config::builder()
        .api_key("test-key")
        .base_url("https://api.test.com/v1")
        .retry_count(3)
        .timeout(Duration::from_secs(120))
        .connect_timeout(Duration::from_secs(15))
        .proxy("http://proxy.test.com:8080")
        .user_agent(HeaderValue::from_static("TestAgent/1.0"))
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
        Some(&HeaderValue::from_static("TestAgent/1.0"))
    );
}

#[test]
fn test_config_builder_defaults() {
    let config = Config::builder()
        .api_key("test-key")
        .base_url("https://api.test.com/v1")
        .build()
        .unwrap();

    assert_eq!(config.retry_count(), 5); // 默认值 
    assert_eq!(config.timeout(), Duration::from_secs(300)); // 默认值
    assert_eq!(config.connect_timeout(), Duration::from_secs(10)); // 默认值
    assert_eq!(config.proxy(), None); // 默认值
    assert_eq!(config.user_agent(), None); // 默认值
}

#[test]
fn test_config_new() {
    let config = Config::new("test-key", "https://api.test.com/v1");
    assert_eq!(config.api_key(), "test-key");
    assert_eq!(config.base_url(), "https://api.test.com/v1");
    assert_eq!(config.retry_count(), 5); // 默认值
    assert_eq!(config.timeout(), Duration::from_secs(300)); // 默认值
    assert_eq!(config.connect_timeout(), Duration::from_secs(10)); // 默认值
    assert_eq!(config.proxy(), None); // 默认值
    assert_eq!(config.user_agent(), None); // 默认值
}

#[test]
fn test_config_setters() {
    let mut config = Config::new("old-key", "https://old-api.com/v1");

    config
        .with_api_key("new-key")
        .with_base_url("https://new-api.com/v1")
        .with_retry_count(2)
        .with_timeout(Duration::from_secs(30))
        .with_connect_timeout(Duration::from_secs(5))
        .with_proxy("http://proxy.example.com:8080")
        .with_user_agent(HeaderValue::from_static("CustomAgent/2.0"));

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
        Some(&HeaderValue::from_static("CustomAgent/2.0"))
    );
}

#[tokio::test]
async fn test_build_openai() {
    let client = Config::builder()
        .api_key("test-key")
        .base_url("https://api.test.com/v1")
        .build_openai()
        .unwrap();

    assert_eq!(client.api_key().await, "test-key");
    assert_eq!(client.base_url().await, "https://api.test.com/v1");
}
