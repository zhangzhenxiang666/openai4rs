use openai4rs::Config;

#[test]
fn test_config_builder() {
    let config = Config::builder()
        .api_key("test-key".to_string())
        .base_url("https://api.test.com/v1".to_string())
        .retry_count(3)
        .timeout_seconds(120)
        .connect_timeout_seconds(15)
        .proxy("http://proxy.test.com:8080")
        .user_agent("TestAgent/1.0")
        .build()
        .unwrap();

    assert_eq!(config.api_key(), "test-key");
    assert_eq!(config.base_url(), "https://api.test.com/v1");
    assert_eq!(config.retry_count(), 3);
    assert_eq!(config.timeout_seconds(), 120);
    assert_eq!(config.connect_timeout_seconds(), 15);
    assert_eq!(
        config.proxy().map(|s| s.as_str()),
        Some("http://proxy.test.com:8080")
    );
    assert_eq!(
        config.user_agent().map(|s| s.as_str()),
        Some("TestAgent/1.0")
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
    assert_eq!(config.timeout_seconds(), 300); // default value
    assert_eq!(config.connect_timeout_seconds(), 10); // default value
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
        .with_timeout_seconds(30)
        .with_connect_timeout_seconds(5)
        .with_proxy("http://proxy.example.com:8080")
        .with_user_agent("CustomAgent/2.0");

    assert_eq!(config.api_key(), "new-key");
    assert_eq!(config.base_url(), "https://new-api.com/v1");
    assert_eq!(config.retry_count(), 2);
    assert_eq!(config.timeout_seconds(), 30);
    assert_eq!(config.connect_timeout_seconds(), 5);
    assert_eq!(
        config.proxy().map(|s| s.as_str()),
        Some("http://proxy.example.com:8080")
    );
    assert_eq!(
        config.user_agent().map(|s| s.as_str()),
        Some("CustomAgent/2.0")
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
