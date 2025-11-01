use openai4rs::OpenAI;

#[tokio::test]
async fn test_openai_setters() {
    let client = OpenAI::new("old-key", "https://old-api.com/v1");

    client.with_api_key("new-key").await;
    client.with_base_url("https://new-api.com/v1").await;
    client.with_retry_count(2).await;
    client.with_timeout_seconds(30).await;
    client.with_connect_timeout_seconds(5).await;
    client.with_proxy("http://proxy.example.com:8080").await;
    client.with_user_agent("CustomAgent/2.0").await;

    let config = client.config().await;

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
