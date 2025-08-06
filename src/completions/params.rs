use derive_builder::Builder;
use serde::Serialize;
use std::collections::HashMap;

/// 创建补全的参数。该结构体代表 OpenAI 补全 API 的请求参数。
/// 请注意，补全 API 是旧版 API，主要用于较旧的模型。对于较新的模型，建议使用聊天补全 API。
#[derive(Debug, Clone, Serialize, Builder)]
#[builder(name = "RequestParamsBuilder")]
#[builder(derive(Debug))]
#[builder(pattern = "owned")]
#[builder(setter(strip_option))]
pub struct RequestParams<'a> {
    /// 要使用的模型的 ID。
    ///
    /// 您可以使用列出模型 API 查看所有可用模型，
    /// 或参阅我们的模型概述以获取其描述。
    pub model: &'a str,

    /// 为其生成补全的提示。
    ///
    /// 请注意，当您提供清晰的指令来定义任务和期望的输出时，
    /// API 的效果最佳。
    pub prompt: &'a str,

    /// 在补全中生成的最大令牌数。
    ///
    /// 您的提示的令牌数加上 `max_tokens` 不能超过
    /// 模型的上下文长度。大多数模型的上下文长度为 2048 个令牌
    /// （最新模型除外，它们支持 4096）。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<i32>,

    /// 使用何种采样温度，介于 0 和 2 之间。
    ///
    /// 较高的值（如 0.8）会使输出更随机，而较低的值（如 0.2）
    /// 会使其更具重点和确定性。
    /// 我们通常建议更改此项或 `top_p`，但不要同时更改两者。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// 一种替代使用温度进行采样的方法，称为核心采样。
    ///
    /// 模型会考虑具有 top_p 概率质量的令牌的结果。
    /// 因此 0.1 意味着只考虑构成前 10% 概率质量的令牌。
    /// 我们通常建议更改此项或 `temperature`，但不要同时更改两者。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// 为每个提示生成多少个补全。
    ///
    /// 请注意，您将根据所有补全中生成的令牌数量付费。
    /// 将 `n` 保持为 `1` 以最小化成本。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,

    /// 是否流式传输部分进度。
    ///
    /// 如果设置，令牌将在可用时作为仅含数据的服务器发送事件发送，
    /// 流以 `data: [DONE]` 消息终止。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// 在 `logprobs` 个最可能的令牌上包含对数概率。
    ///
    /// 设置为 0 以禁用返回任何对数概率。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<i32>,

    /// 除了补全之外，还回显提示。
    ///
    /// 这对于调试和理解模型的行为很有用。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub echo: Option<bool>,

    /// API 将停止生成更多令牌的最多 4 个序列。返回的文本将不包含停止序列。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// 介于 -2.0 和 2.0 之间的数字。正值会根据新词是否出现在目前为止的文本中来对其进行惩罚，
    /// 从而增加模型谈论新话题的可能性。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// 介于 -2.0 和 2.0 之间的数字。正值会根据新词在文本中至今的现有频率对其进行惩罚，
    /// 从而降低模型逐字重复同一行的可能性。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// 在服务器端生成 `best_of` 个补全，并返回"最佳"
    /// （每个令牌的对数概率最高的那个）。
    ///
    /// 结果无法流式传输。当与 `n` 一起使用时，`best_of` 控制
    /// 候选补全的数量，`n` 指定要返回的数量。
    /// `best_of` 必须大于或等于 `n`。
    /// 注意：由于此参数会生成许多补全，因此会很快
    /// 消耗您的令牌配额。请谨慎使用，并确保您为 `max_tokens`
    /// 和 `stop` 设置了合理的参数。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub best_of: Option<i32>,

    /// 修改指定词出现在补全中的可能性。
    ///
    /// 接受一个 JSON 对象，该对象将词（由其在分词器中的词 ID 指定）
    /// 映射到 -100 到 100 之间的关联偏差值。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,

    /// 代表您的最终用户的唯一标识符，可以帮助 OpenAI
    /// 监控和检测滥用行为。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// 随请求发送额外的标头。
    /// 此字段不会在请求正文中序列化。
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_headers: Option<HashMap<String, serde_json::Value>>,

    /// 向请求添加额外的查询参数。
    /// 此字段不会在请求正文中序列化。
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_query: Option<HashMap<String, serde_json::Value>>,

    /// 向请求添加额外的 JSON 属性。
    /// 此字段不会在请求正文中序列化。
    #[builder(default)]
    #[serde(skip_serializing)]
    pub extra_body: Option<HashMap<String, serde_json::Value>>,

    /// HTTP 请求重试次数，覆盖客户端的全局设置。
    /// 此字段不会在请求正文中序列化。
    #[builder(default)]
    #[serde(skip_serializing)]
    pub retry_count: Option<u32>,

    /// HTTP 请求超时时间（秒），覆盖客户端的全局设置。
    /// 此字段不会在请求正文中序列化。
    #[builder(default)]
    #[serde(skip_serializing)]
    pub timeout_seconds: Option<u64>,

    /// HTTP 请求 User-Agent，覆盖客户端的全局设置。
    /// 此字段不会在请求正文中序列化。
    #[builder(default)]
    #[serde(skip_serializing)]
    pub user_agent: Option<String>,
}

pub fn completions_request<'a>(model: &'a str, prompt: &'a str) -> RequestParamsBuilder<'a> {
    RequestParamsBuilder::create_empty()
        .model(model)
        .prompt(prompt)
}

pub trait IntoRequestParams<'a> {
    fn into_request_params(self) -> RequestParams<'a>;
}

impl<'a> IntoRequestParams<'a> for RequestParams<'a> {
    fn into_request_params(self) -> RequestParams<'a> {
        self
    }
}

impl<'a> IntoRequestParams<'a> for RequestParamsBuilder<'a> {
    fn into_request_params(self) -> RequestParams<'a> {
        self.build().unwrap()
    }
}
