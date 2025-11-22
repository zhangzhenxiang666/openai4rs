use super::request::RequestSpec;
use crate::config::Config;
use crate::error::{OpenAIError, ProcessingError};
use crate::service::executor::HttpExecutor;
use crate::service::request::Request;
use eventsource_stream::{Event, EventStreamError, Eventsource};
use futures::StreamExt;
use std::any::type_name;
use std::sync::{RwLockReadGuard, RwLockWriteGuard};
use tokio_stream::wrappers::ReceiverStream;

/// 用于处理流事件的结果类型。
///
/// 此枚举表示处理流事件时可能出现的结果：
/// - Skip: 应该忽略该事件（例如，空数据）
/// - Data: 事件包含应转发的有效数据
/// - Done: 流已完成
/// - Error: 处理事件时发生错误
enum SseEventResult<T>
where
    T: serde::de::DeserializeOwned,
{
    /// 跳过此事件（例如，空数据）
    Skip,
    /// 从事件中提取的有效数据
    Data(T),
    /// 流已完成
    Done,
    /// 处理事件时发生错误
    Error(OpenAIError),
}

/// 抽象底层HTTP服务的传输层。
///
/// 此层为发送HTTP请求提供简化的接口，
/// 将实际执行委托给 `HttpExecutor`。它处理
/// 响应处理，包括JSON反序列化和流
/// 响应处理。
///
/// 传输层负责：
/// - 将原始HTTP响应转换为强类型对象
/// - 使用服务器发送事件（SSE）处理流响应
/// - 管理请求/响应生命周期
pub(crate) struct InnerHttp {
    /// 负责发送请求的底层HTTP执行器
    executor: HttpExecutor,
}

impl InnerHttp {
    /// 使用给定的配置创建一个新的 `InnerHttp`。
    ///
    /// # 参数
    /// * `config` - 主OpenAI客户端配置
    ///
    /// # 返回值
    /// 新的InnerHttp实例
    pub fn new(config: Config) -> InnerHttp {
        InnerHttp {
            executor: HttpExecutor::new(config),
        }
    }

    /// 获取对配置的只读访问权限。
    pub fn config_read(&self) -> RwLockReadGuard<'_, Config> {
        self.executor.config_read()
    }

    /// 获取对配置的写入访问权限。
    pub fn config_write(&self) -> RwLockWriteGuard<'_, Config> {
        self.executor.config_write()
    }

    /// 使用JSON负载发送POST请求并使用HttpParams反序列化响应。
    ///
    /// 此方法发送一个POST请求并自动将
    /// JSON响应反序列化为指定类型。
    ///
    /// # 参数
    /// * `params` - 包含所有必要请求参数的HttpParams结构
    ///
    /// # 类型参数
    /// * `U` - 用于生成URL的函数类型，返回一个String
    /// * `F` - 用于构建请求的函数类型
    /// * `T` - 实现DeserializeOwned的预期响应类型
    ///
    /// # 返回值
    /// 包含反序列化响应对象或OpenAIError的Result
    pub async fn post_json<U, F, T>(&self, params: RequestSpec<U, F>) -> Result<T, OpenAIError>
    where
        U: FnOnce(&Config) -> String,
        F: FnOnce(&Config, Request) -> Request,
        T: serde::de::DeserializeOwned,
    {
        let res = self.executor.post(params).await?;

        let status = res.status();
        let url = res.url().clone();

        res.json().await.map_err(|e| {
            ProcessingError::JsonDeserialization {
                error: e,
                target_type: type_name::<T>().to_string(),
                status_code: Some(status.as_u16()),
                url: Some(url.to_string()),
            }
            .into()
        })
    }

    /// 使用HttpParams发送GET请求并反序列化JSON响应。
    ///
    /// 此方法发送一个GET请求并自动将
    /// JSON响应反序列化为指定类型。
    ///
    /// # 参数
    /// * `params` - 包含所有必要请求参数的HttpParams结构
    ///
    /// # 类型参数
    /// * `U` - 用于生成URL的函数类型，返回一个String
    /// * `F` - 用于构建请求的函数类型
    /// * `T` - 实现DeserializeOwned的预期响应类型
    ///
    /// # 返回值
    /// 包含反序列化响应对象或OpenAIError的Result
    pub async fn get_json<U, F, T>(&self, params: RequestSpec<U, F>) -> Result<T, OpenAIError>
    where
        U: FnOnce(&Config) -> String,
        F: FnOnce(&Config, Request) -> Request,
        T: serde::de::DeserializeOwned,
    {
        let res = self.executor.get(params).await?;

        let status = res.status();
        let url = res.url().clone();

        res.json().await.map_err(|e| {
            ProcessingError::JsonDeserialization {
                error: e,
                target_type: type_name::<T>().to_string(),
                status_code: Some(status.as_u16()),
                url: Some(url.to_string()),
            }
            .into()
        })
    }

    /// 使用HttpParams发送POST请求并期望流式JSON响应。
    ///
    /// 此方法发送一个POST请求并处理流式响应
    /// 使用服务器发送事件（SSE）。它返回反序列化
    /// 响应块的流。
    ///
    /// # 参数
    /// * `params` - 包含所有必要请求参数的HttpParams结构
    ///
    /// # 类型参数
    /// * `U` - 用于生成URL的函数类型，返回一个String
    /// * `F` - 用于构建请求的函数类型
    /// * `T` - 实现DeserializeOwned的预期响应块类型
    ///
    /// # 返回值
    /// 包含响应块流或OpenAIError的Result
    pub async fn post_json_stream<U, F, T>(
        &self,
        params: RequestSpec<U, F>,
    ) -> Result<tokio_stream::wrappers::ReceiverStream<Result<T, OpenAIError>>, OpenAIError>
    where
        U: FnOnce(&Config) -> String,
        F: FnOnce(&Config, Request) -> Request,
        T: serde::de::DeserializeOwned + Send + 'static,
    {
        let res = self.executor.post(params).await?;
        let mut event_stream = res.bytes_stream().eventsource();
        let (tx, rx) = tokio::sync::mpsc::channel(32);

        tokio::spawn(async move {
            while let Some(event_result) = event_stream.next().await {
                let process_result = Self::process_stream_event(event_result);
                match process_result {
                    SseEventResult::Skip => continue,
                    SseEventResult::Data(chunk) => {
                        if tx.send(Ok(chunk)).await.is_err() {
                            break;
                        }
                    }
                    SseEventResult::Done => break,
                    SseEventResult::Error(error) => {
                        if tx.send(Err(error)).await.is_err() {
                            break;
                        }
                    }
                }
            }
            drop(tx);
        });

        Ok(ReceiverStream::new(rx))
    }

    /// 处理来自SSE流的流式事件。
    ///
    /// 此方法处理解析和处理单个事件
    /// 来自服务器发送事件流，将它们转换为ProcessEventResult
    /// 变体。
    ///
    /// # 参数
    /// * `event_result` - 事件流的结果（事件或错误）
    ///
    /// # 类型参数
    /// * `T` - 实现DeserializeOwned的预期响应块类型
    ///
    /// # 返回值
    /// 指示如何处理此事件的ProcessEventResult
    fn process_stream_event<T>(
        event_result: Result<Event, EventStreamError<reqwest::Error>>,
    ) -> SseEventResult<T>
    where
        T: serde::de::DeserializeOwned + Send + 'static,
    {
        match event_result {
            Ok(event) => {
                // 如果数据为空就跳过这个事件
                if event.data.is_empty() {
                    return SseEventResult::Skip;
                }

                // 检查sse完成标志
                if event.data == "[DONE]" {
                    SseEventResult::Done
                } else {
                    // 尝试将事件数据反序列化为预期类型
                    match serde_json::from_str::<T>(&event.data) {
                        Ok(chunk) => SseEventResult::Data(chunk),
                        Err(_) => SseEventResult::Error(
                            ProcessingError::Conversion {
                                raw: event.data,
                                target_type: type_name::<T>().to_string(),
                            }
                            .into(),
                        ),
                    }
                }
            }
            Err(e) => SseEventResult::Error(OpenAIError::from_eventsource_stream_error(e)),
        }
    }

    /// 更新内部HTTP客户端配置。
    ///
    /// 此方法触发底层HTTP客户端的重建
    /// 以及任何更新的配置设置。
    pub fn refresh_client(&self) {
        self.executor.rebuild_reqwest_client();
    }
}
