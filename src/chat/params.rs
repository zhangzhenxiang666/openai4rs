use super::types::*;
use crate::common::types::ServiceTier;
use derive_builder::Builder;
use serde::Serialize;
use std::collections::HashMap;

/// 用于为聊天对话创建模型响应的参数。
///
/// 该结构体代表 OpenAI 聊天补全 API 的请求参数，
/// 支持文本生成、视觉和音频功能。参数支持
/// 可能因所使用的模型而异，特别是对于较新的推理模型。
#[derive(Debug, Clone, Serialize, Builder)]
#[builder(name = "RequestParamsBuilder")]
#[builder(derive(Debug))]
#[builder(pattern = "owned")]
#[builder(setter(strip_option))]
pub struct RequestParams<'a> {
    /// 用于生成响应的模型 ID，例如 `gpt-4o` 或 `o1`。
    ///
    /// OpenAI 提供具有不同功能、
    /// 性能特点和价位的多种模型。
    pub model: &'a str,

    /// 构成迄今为止对话的消息列表。
    ///
    /// 根据您使用的模型，支持不同的消息类型（模态），
    /// 例如文本、图像和音频。
    pub messages: &'a Vec<ChatCompletionMessageParam>,

    /// 介于 -2.0 和 2.0 之间的数字。正值会根据新词在文本中至今的现有频率对其进行惩罚，
    /// 从而降低模型逐字重复同一行的可能性。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// 修改指定词出现在补全中的可能性。
    ///
    /// 接受一个 JSON 对象，该对象将词（由其在分词器中的词 ID 指定）
    /// 映射到 -100 到 100 之间的关联偏差值。偏差会在采样前
    /// 添加到模型生成的对数中。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<HashMap<String, i32>>,

    /// 是否返回输出词的对数概率。
    ///
    /// 如果为 true，则返回 `message` 的 `content` 中
    /// 每个输出词的对数概率。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// 您希望模型生成的输出类型。
    ///
    /// 大多数模型能够生成文本，这是默认设置：`["text"]`。
    /// `gpt-4o-audio-preview` 模型还可以生成音频。要同时请求
    /// 文本和音频响应，请使用：`["text", "audio"]`。
    #[builder(default)]
    #[serde(skip_serializing_if = "skip_if_option_vec_empty")]
    pub modalities: Option<Vec<Modality>>,

    /// 可为补全生成的令牌数量的上限，
    ///
    /// 包括可见的输出令牌和推理令牌。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<i32>,

    /// 在聊天补全中可以生成的最大令牌数。
    ///
    /// 此值可用于控制通过 API 生成的文本的成本。
    /// 适用于 o1 系列模型。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    #[deprecated(note = "Use `max_completion_tokens` instead")]
    pub max_tokens: Option<i32>,

    /// 可以附加到对象的 16 个键值对的集合。
    ///
    /// 这对于以结构化格式存储有关对象的附加信息很有用。
    /// 键的最大长度为 64 个字符，值的最大长度为 512 个字符。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,

    /// 在工具使用期间是否启用并行函数调用。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,

    /// 介于 -2.0 和 2.0 之间的数字。正值会根据新词是否出现在目前为止的文本中来对其进行惩罚，
    /// 从而增加模型谈论新话题的可能性。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// 为每个输入消息生成多少个聊天补全选项。
    ///
    /// 请注意，您将根据所有选项中生成的令牌数量付费。
    /// 将 `n` 保持为 `1` 以最小化成本。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<i32>,

    /// 一种替代使用温度进行采样的方法，称为核心采样。
    ///
    /// 模型会考虑具有 top_p 概率质量的令牌的结果。
    /// 因此 0.1 意味着只考虑构成前 10% 概率质量的令牌。
    /// 我们通常建议更改此参数或 `temperature`，但不要同时更改两者。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// 如果设置为 true，模型响应数据将在生成时使用服务器发送事件
    /// 流式传输到客户端。
    ///
    /// 有关如何处理流式事件的更多信息，请参阅流式响应指南。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// 是否存储此聊天补全请求的输出以用于
    /// 模型蒸馏或评估产品。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,

    /// API 将停止生成更多令牌的最多 4 个序列。
    ///
    /// 返回的文本将不包含停止序列。
    /// **注意**：此字段名称在您的结构中显示为 `send`，但很可能应该是 `stop`。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub send: Option<i32>,

    /// 使用什么采样温度，介于 0 和 2 之间。
    ///
    /// 较高的值（如 0.8）会使输出更随机，而较低的值（如 0.2）
    /// 会使其更具重点和确定性。我们通常建议更改此参数或 `top_p`，
    /// 但不要同时更改两者。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// 代表您的最终用户的唯一标识符，可以帮助 OpenAI
    /// 监控和检测滥用行为。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// 一个介于 0 和 20 之间的整数，指定在每个令牌位置返回的最可能令牌的数量，
    /// 每个令牌都具有关联的对数概率。
    /// 如果使用此参数，`logprobs` 必须设置为 `true`。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<i32>,

    /// 静态预测的输出内容，例如正在重新生成的
    /// 文本文件的内容。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<ChatCompletionPredictionContentParam>,

    /// **仅限 o 系列模型** - 限制推理模型的推理工作量。
    ///
    /// 当前支持的值为 `low`、`medium` 和 `high`。减少推理工作量
    /// 可以加快响应速度，并减少响应中用于推理的令牌数量。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,

    /// 指定用于处理请求的延迟等级。
    ///
    /// 此参数与订阅了规模等级服务的客户相关。
    /// - 如果设置为 'auto'，并且项目启用了规模等级，系统将
    ///   利用规模等级积分，直到用完为止。
    /// - 如果设置为 'default'，请求将使用默认服务
    ///   等级处理，其正常运行时间 SLA 较低，且没有延迟保证。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<ServiceTier>,

    /// 模型可能调用的工具列表。目前，仅支持函数作为工具。
    ///
    /// 使用此参数提供模型可能为其生成 JSON 输入的函数列表。
    /// 最多支持 128 个函数。
    #[builder(default)]
    #[serde(skip_serializing_if = "skip_if_option_vec_empty")]
    pub tools: Option<Vec<ChatCompletionToolParam>>,

    /// 控制模型调用哪个（或哪些）工具。
    ///
    /// - `none` 表示模型不会调用任何工具，而是生成一条消息。
    /// - `auto` 表示模型可以在生成消息或调用一个或多个工具之间进行选择。
    /// - `required` 表示模型必须调用一个或多个工具。
    /// - 指定特定工具会强制模型调用该工具。
    ///
    /// 当不存在工具时，默认为 `none`。如果存在工具，则默认为 `auto`。
    #[builder(default)]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

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

pub fn chat_request<'a>(
    model: &'a str,
    messages: &'a Vec<ChatCompletionMessageParam>,
) -> RequestParamsBuilder<'a> {
    RequestParamsBuilder::create_empty()
        .model(model)
        .messages(messages)
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

fn skip_if_option_vec_empty<T>(opt: &Option<Vec<T>>) -> bool
where
    T: std::fmt::Debug,
{
    opt.as_ref().is_none_or(Vec::is_empty)
}
