use super::request::{Request, RequestBuilder, RequestSpec};
use crate::common::types::RetryCount;
use crate::config::Config;
use crate::error::{ApiError, ApiErrorKind, OpenAIError, RequestError};
use crate::utils::traits::AsyncFrom;
use rand::Rng;
use reqwest::{Client, Response};
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};
use std::time::Duration;

/// 处理实际发送HTTP请求的HTTP请求执行器。
///
/// 该组件负责：
/// - 构建和维护底层reqwest HTTP客户端
/// - 使用重试逻辑执行HTTP请求
/// - 处理请求/响应生命周期，包括错误处理
///
/// 执行器对reqwest客户端使用读写锁，以允许并发读取，
/// 同时确保配置更改时的线程安全更新。
pub(crate) struct HttpExecutor {
    config: RwLock<Config>,
    reqwest_client: RwLock<Client>,
}

impl HttpExecutor {
    pub fn new(config: Config) -> HttpExecutor {
        let reqwest_client = config.http().build_reqwest_client();
        HttpExecutor {
            config: RwLock::new(config),
            reqwest_client: RwLock::new(reqwest_client),
        }
    }

    #[inline]
    pub fn config_read(&self) -> RwLockReadGuard<'_, Config> {
        self.config.read().expect("Failed to acquire read lock on config. This indicates a serious internal error, possibly due to a poisoned RwLock.")
    }

    #[inline]
    pub fn config_write(&self) -> RwLockWriteGuard<'_, Config> {
        self.config.write().expect("Failed to acquire write lock on config. This indicates a serious internal error, possibly due to a poisoned RwLock.")
    }

    pub fn rebuild_reqwest_client(&self) {
        let new_client = {
            let config_guard = self.config_read();
            config_guard.http().build_reqwest_client()
        };
        let mut client_guard = self.client_write();
        *client_guard = new_client;
    }

    /// 根据请求参数发送post请求
    pub async fn post<U, F>(&self, params: RequestSpec<U, F>) -> Result<Response, OpenAIError>
    where
        U: FnOnce(&Config) -> String,
        F: FnOnce(&Config, Request) -> Request,
    {
        self.send(reqwest::Method::POST, params).await
    }

    /// 根据请求参数发送get请求
    pub async fn get<U, F>(&self, params: RequestSpec<U, F>) -> Result<Response, OpenAIError>
    where
        U: FnOnce(&Config) -> String,
        F: FnOnce(&Config, Request) -> Request,
    {
        self.send(reqwest::Method::GET, params).await
    }
}

impl HttpExecutor {
    #[inline]
    fn client_read(&self) -> RwLockReadGuard<'_, Client> {
        self.reqwest_client.read().expect("Failed to acquire read lock on reqwest_client. This indicates a serious internal error, possibly due to a poisoned RwLock.")
    }

    #[inline]
    pub fn client_write(&self) -> RwLockWriteGuard<'_, Client> {
        self.reqwest_client.write().expect("Failed to acquire write lock on reqwest_client during rebuild. This indicates a serious internal error, possibly due to a poisoned RwLock.")
    }

    async fn send<U, F>(
        &self,
        method: reqwest::Method,
        params: RequestSpec<U, F>,
    ) -> Result<Response, OpenAIError>
    where
        U: FnOnce(&Config) -> String,
        F: FnOnce(&Config, Request) -> Request,
    {
        let client = self.client_read().clone();

        let (retry_count, request) = {
            let config_guard = self.config_read();

            let mut request = Request::new(method, (params.url_fn)(&config_guard));

            request = (params.builder_fn)(&config_guard, request);

            let mut request_builder = RequestBuilder::new(request);

            HttpExecutor::apply_global_http_settings(&config_guard, &mut request_builder);

            request = request_builder.take();

            let retry_count = match request.extensions().get::<RetryCount>() {
                Some(retry) if retry.0 != 0 => retry.0,
                _ => config_guard.retry_count(),
            };

            (retry_count, request)
        };

        HttpExecutor::send_with_retries(request, retry_count as u32, client).await
    }

    fn apply_global_http_settings(config: &Config, request_builder: &mut RequestBuilder) {
        // 仅在本地未设置时才应用全局头
        config.http().headers().iter().for_each(|(k, v)| {
            if !request_builder.has_header(k) {
                request_builder.header(k, v.clone());
            }
        });

        // 仅在本地未设置时才应用全局主体字段
        config.http().bodys().iter().for_each(|(k, v)| {
            if !request_builder.has_body_field(k) {
                request_builder.body_field(k, v.clone());
            }
        });
    }

    async fn send_with_retries(
        request: Request,
        retry_count: u32,
        client: reqwest::Client,
    ) -> Result<Response, OpenAIError> {
        let mut attempts = 0;
        let max_attempts = retry_count.max(1);

        loop {
            attempts += 1;

            // Convert to reqwest RequestBuilder
            let request_builder = request.to_reqwest(&client);

            match request_builder.send().await {
                Ok(response) => {
                    // Check for retry-after header from the server
                    let retry_after = response
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|v| v.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(Duration::from_secs);

                    if response.status().is_success() {
                        return Ok(response);
                    } else {
                        let api_error = ApiError::async_from(response).await;

                        // Check if we should retry or return error with interceptors applied
                        if attempts >= max_attempts || !api_error.is_retryable() {
                            return Err(api_error.into());
                        }

                        tracing::debug!(
                            "Attempt {}/{}: Retrying after API error: {:?}",
                            attempts,
                            max_attempts,
                            api_error
                        );
                        tokio::time::sleep(calculate_retry_delay(
                            attempts,
                            &api_error.kind,
                            retry_after,
                        ))
                        .await;
                    }
                }
                Err(e) => {
                    let request_error: RequestError = e.into();

                    // Check if we should retry or return error with interceptors applied
                    if attempts >= max_attempts || !request_error.is_retryable() {
                        return Err(request_error.into());
                    }

                    tracing::debug!(
                        "Attempt {}/{}: Retrying after request error: {:?}",
                        attempts,
                        max_attempts,
                        request_error
                    );
                    tokio::time::sleep(calculate_retry_delay_for_request_error(
                        attempts,
                        &request_error,
                    ))
                    .await;
                }
            }
        }
    }
}

const API_ERROR_DEFAULT_BASE_DELAY_MS: u64 = 500;
const API_ERROR_INTERNAL_SERVER_BASE_DELAY_MS: u64 = 1000;
const API_ERROR_RATE_LIMIT_BASE_DELAY_MS: u64 = 5000;
const API_ERROR_MAX_DELAY_MS: u64 = 30_000;

const REQUEST_ERROR_DEFAULT_BASE_DELAY_MS: u64 = 100;
const REQUEST_ERROR_CONNECTION_BASE_DELAY_MS: u64 = 200;
const REQUEST_ERROR_MAX_DELAY_MS: u64 = 10_000;

const RETRY_AFTER_JITTER_MS: u64 = 1000;

/// 根据错误类型计算重试前的适当延迟。
///
/// 此函数实现带有抖动的指数退避策略，
/// 并对速率限制错误和服务器错误进行特殊处理。
///
/// # 参数
/// * `attempt` - 当前尝试次数（从1开始）
/// * `error_kind` - 发生的API错误类型
/// * `retry_after` - 服务器指定的可选重试延迟
///
/// # 返回值
/// 重试前等待的持续时间
fn calculate_retry_delay(
    attempt: u32,
    error_kind: &ApiErrorKind,
    retry_after: Option<Duration>,
) -> Duration {
    // 如果服务器指定了重试延迟，使用该延迟并添加抖动
    if let Some(duration) = retry_after {
        let jitter = Duration::from_millis(rand::thread_rng().gen_range(0..RETRY_AFTER_JITTER_MS));
        return duration + jitter;
    }

    // 基础延迟因错误类型而异
    let base_delay_ms = match error_kind {
        ApiErrorKind::RateLimit => API_ERROR_RATE_LIMIT_BASE_DELAY_MS,
        ApiErrorKind::InternalServer => API_ERROR_INTERNAL_SERVER_BASE_DELAY_MS,
        _ => API_ERROR_DEFAULT_BASE_DELAY_MS,
    };

    // 指数退避：base_delay * 2^(attempt-1)
    let delay_ms = base_delay_ms.saturating_mul(2u64.pow(attempt - 1));
    // 将延迟限制在最大值内
    let base_delay = Duration::from_millis(delay_ms.min(API_ERROR_MAX_DELAY_MS));

    // 添加0-10%的抖动以防止雷鸣般涌入
    let jitter_percent = rand::thread_rng().gen_range(0..10);
    let jitter_ms = (base_delay.as_millis() as u64 * jitter_percent) / 100;
    base_delay + Duration::from_millis(jitter_ms)
}

/// 根据请求错误计算重试前的适当延迟。
///
/// 此函数为网络级请求错误实现带有抖动的指数退避策略。
///
/// # 参数
/// * `attempt` - 当前尝试次数（从1开始）
/// * `error` - 发生的请求错误
///
/// # 返回值
/// 重试前等待的持续时间
fn calculate_retry_delay_for_request_error(attempt: u32, error: &RequestError) -> Duration {
    // 基础延迟因错误类型而异
    let base_delay_ms = match error {
        RequestError::Timeout(_) => REQUEST_ERROR_DEFAULT_BASE_DELAY_MS,
        RequestError::Connection(_) => REQUEST_ERROR_CONNECTION_BASE_DELAY_MS,
        _ => REQUEST_ERROR_DEFAULT_BASE_DELAY_MS,
    };

    // 指数退避：base_delay * 2^(attempt-1)
    let delay_ms = base_delay_ms.saturating_mul(2u64.pow(attempt - 1));
    // 将延迟限制在最大值内
    let base_delay = Duration::from_millis(delay_ms.min(REQUEST_ERROR_MAX_DELAY_MS));

    // 添加0-10%的抖动以防止雷鸣般涌入
    let jitter_percent = rand::thread_rng().gen_range(0..10);
    let jitter_ms = (base_delay.as_millis() as u64 * jitter_percent) / 100;
    base_delay + Duration::from_millis(jitter_ms)
}
