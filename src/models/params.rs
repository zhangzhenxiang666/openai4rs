use derive_builder::Builder;
use serde::Serialize;
use std::collections::HashMap;

/// 用于列出模型的参数。
///
/// 该结构体代表 OpenAI 模型 API 的请求参数。
/// 它允许您查询可用模型及其功能。
#[derive(Debug, Clone, Serialize, Builder)]
#[builder(name = "RequestParamsBuilder")]
#[builder(derive(Debug))]
#[builder(pattern = "owned")]
#[builder(setter(strip_option))]
pub struct RequestParams {
    /// 随请求发送额外的标头。
    ///
    /// 此字段不会在请求正文中序列化。
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_headers: Option<HashMap<String, serde_json::Value>>,

    /// 向请求添加额外的查询参数。
    ///
    /// 此字段不会在请求正文中序列化。
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_query: Option<HashMap<String, serde_json::Value>>,

    /// 向请求添加额外的 JSON 属性。
    ///
    /// 此字段不会在请求正文中序列化。
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_body: Option<HashMap<String, serde_json::Value>>,

    /// HTTP 请求重试次数，覆盖客户端的全局设置。
    ///
    /// 此字段不会在请求正文中序列化。
    #[builder(default)]
    #[serde(skip_serializing)]
    pub retry_count: Option<u32>,

    /// HTTP 请求超时时间（秒），覆盖客户端的全局设置。
    ///
    /// 此字段不会在请求正文中序列化。
    #[builder(default)]
    #[serde(skip_serializing)]
    pub timeout_seconds: Option<u64>,

    /// HTTP 请求 User-Agent，覆盖客户端的全局设置。
    ///
    /// 此字段不会在请求正文中序列化。
    #[builder(default)]
    #[serde(skip_serializing)]
    pub user_agent: Option<String>,
}

pub fn models_request() -> RequestParamsBuilder {
    RequestParamsBuilder::create_empty()
}

pub trait IntoRequestParams {
    fn into_request_params(self) -> RequestParams;
}

impl IntoRequestParams for RequestParams {
    fn into_request_params(self) -> RequestParams {
        self
    }
}

impl IntoRequestParams for RequestParamsBuilder {
    fn into_request_params(self) -> RequestParams {
        self.build().unwrap()
    }
}
