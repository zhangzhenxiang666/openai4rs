use super::request::{Request, RequestBuilder, RequestSpec};
use crate::Config;
use crate::common::types::RetryCount;
use crate::error::{ApiError, ApiErrorKind, OpenAIError, RequestError};
use crate::utils::traits::AsyncFrom;
use rand::Rng;
use reqwest::{Client, Response};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

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
    /// 主OpenAI客户端配置。
    ///
    /// 用于确定重试次数和其他客户端特定设置。
    config: Arc<RwLock<Config>>,

    /// 包装在RwLock中的底层reqwest HTTP客户端。
    ///
    /// 这允许多个并发请求，同时确保配置更改时的线程安全更新。
    reqwest_client: RwLock<Client>,
}

impl HttpExecutor {
    /// 使用给定配置创建一个新的HttpExecutor。
    ///
    /// # 参数
    /// * `config` - 主OpenAI客户端配置
    ///
    /// # 返回值
    /// 新的HttpExecutor实例
    pub fn new(config: Config) -> HttpExecutor {
        let reqwest_client = config.http().build_reqwest_client();
        HttpExecutor {
            config: Arc::new(RwLock::new(config)),
            reqwest_client: RwLock::new(reqwest_client),
        }
    }

    /// 返回包装在Arc<RwLock>中的内部配置的克隆。
    ///
    /// 这允许访问当前配置以构建请求。
    pub(crate) fn config(&self) -> Arc<RwLock<Config>> {
        self.config.clone()
    }

    /// 根据当前配置重建内部的`reqwest::Client`。
    ///
    /// 当HTTP配置更改时应调用此方法，
    /// 例如当代理设置或超时值更新时。
    pub async fn rebuild_reqwest_client(&self) {
        let new_client = {
            let config_guard = self.config.read().await;
            config_guard.http().build_reqwest_client()
        };
        let mut client_guard = self.reqwest_client.write().await;
        *client_guard = new_client;
    }

    /// 使用HttpParams发送POST请求并返回原始HTTP响应。
    ///
    /// 此方法处理完整的请求生命周期，包括：
    /// - 使用提供的函数构建请求
    /// - 使用重试逻辑执行请求
    /// - 处理错误和重试
    ///
    /// # 参数
    /// * `params` - 包含所有必要请求参数的HttpParams结构
    ///
    /// # 类型参数
    /// * `U` - 用于生成URL的函数类型，返回一个String
    /// * `F` - 用于构建请求的函数类型
    ///
    /// # 返回值
    /// 包含原始HTTP响应或OpenAIError的Result
    pub async fn post<U, F>(&self, params: RequestSpec<U, F>) -> Result<Response, OpenAIError>
    where
        U: FnOnce(&Config) -> String,
        F: FnOnce(&Config, Request) -> Request,
    {
        // Snapshot client and config-derived values to avoid holding locks across await
        let client = {
            let client_guard = self.reqwest_client.read().await;
            client_guard.clone()
        };

        let (retry_count, request) = {
            let config_guard = self.config.read().await;

            let mut request = Request::new(reqwest::Method::POST, (params.url_fn)(&config_guard));

            request = (params.builder_fn)(&config_guard, request);

            let mut request_builder = RequestBuilder::new(request);

            HttpExecutor::apply_global_http_settings(&config_guard, &mut request_builder);

            request = request_builder.take();

            let retry_count = match request.extensions().get::<RetryCount>() {
                Some(retry) => {
                    if retry.0 != 0 {
                        retry.0
                    } else {
                        config_guard.retry_count()
                    }
                }
                None => config_guard.retry_count(),
            };

            (retry_count, request)
        };

        HttpExecutor::send_with_retries(request, retry_count as u32, client).await
    }

    /// 使用HttpParams发送GET请求并返回原始HTTP响应。
    ///
    /// 此方法处理完整的请求生命周期，包括：
    /// - 使用提供的函数构建请求
    /// - 使用重试逻辑执行请求
    /// - 处理错误和重试
    ///
    /// # 参数
    /// * `params` - 包含所有必要请求参数的HttpParams结构
    ///
    /// # 类型参数
    /// * `U` - 用于生成URL的函数类型，返回一个String
    /// * `F` - 用于构建请求的函数类型
    ///
    /// # 返回值
    /// 包含原始HTTP响应或OpenAIError的Result
    pub async fn get<U, F>(&self, params: RequestSpec<U, F>) -> Result<Response, OpenAIError>
    where
        U: FnOnce(&Config) -> String,
        F: FnOnce(&Config, Request) -> Request,
    {
        // Snapshot client and config-derived values to avoid holding locks across await
        let client = {
            let client_guard = self.reqwest_client.read().await;
            client_guard.clone()
        };

        let (retry_count, request) = {
            let config_guard = self.config.read().await;

            let mut request = Request::new(reqwest::Method::GET, (params.url_fn)(&config_guard));

            request = (params.builder_fn)(&config_guard, request);

            let mut request_builder = RequestBuilder::new(request);

            HttpExecutor::apply_global_http_settings(&config_guard, &mut request_builder);

            request = request_builder.take();

            let retry_count = match request.extensions().get::<RetryCount>() {
                Some(retry) => {
                    if retry.0 != 0 {
                        retry.0
                    } else {
                        config_guard.retry_count()
                    }
                }
                None => config_guard.retry_count(),
            };

            (retry_count, request)
        };

        HttpExecutor::send_with_retries(request, retry_count as u32, client).await
    }
}

impl HttpExecutor {
    /// 将全局HTTP设置（头、查询参数、主体字段）应用到请求构建器
    /// 仅在本地未设置时才应用全局设置（本地具有更高优先级）
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

// --- 重试逻辑的实用函数（从client/http.rs迁移） ---

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
        let jitter = Duration::from_millis(rand::thread_rng().gen_range(0..1000));
        return duration + jitter;
    }

    // 基础延迟因错误类型而异
    let base_delay_ms = match error_kind {
        ApiErrorKind::RateLimit => 5000,      // 速率限制为5秒
        ApiErrorKind::InternalServer => 1000, // 服务器错误为1秒
        _ => 500,                             // 其他错误为0.5秒
    };

    // 指数退避：base_delay * 2^(attempt-1)
    let delay_ms = base_delay_ms * 2u64.pow(attempt - 1);
    // 将延迟限制为30秒
    let base_delay = Duration::from_millis(delay_ms.min(30_000));

    // 添加0-10%的抖动以防止雷鸣般涌入
    let jitter_ms = (base_delay.as_millis() as u64 * (rand::thread_rng().gen_range(0..10))) / 100;
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
    let base_delay = match error {
        RequestError::Timeout(_) => 100,    // 超时为100ms
        RequestError::Connection(_) => 200, // 连接错误为200ms
        _ => 100,                           // 其他错误为100ms
    };

    // 指数退避：base_delay * 2^(attempt-1)
    let delay_ms = base_delay * 2u64.pow(attempt - 1);
    // 将延迟限制为10秒
    let base_delay = Duration::from_millis(delay_ms.min(10_000));

    // 添加0-10%的抖动以防止雷鸣般涌入
    let jitter_ms = (base_delay.as_millis() as u64 * (rand::random::<u64>() % 10)) / 100;
    base_delay + Duration::from_millis(jitter_ms)
}
