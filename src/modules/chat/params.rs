use super::types::{
    ChatCompletionMessageParam, ChatCompletionPredictionContentParam, ChatCompletionToolParam,
    Modality, ReasoningEffort, ToolChoice,
};
use crate::common::types::{InParam, JsonBody, RetryCount, ServiceTier, Timeout};
use http::{
    HeaderValue,
    header::{IntoHeaderName, USER_AGENT},
};
use serde_json::Value;
use std::{collections::HashMap, time::Duration};

pub struct ChatParam {
    inner: InParam,
}

impl ChatParam {
    #[doc = include_str!("../../docs/chat_param.md")]
    pub fn new(model: &str, messages: &Vec<ChatCompletionMessageParam>) -> Self {
        let mut inner = InParam::new();
        inner.body = Some(JsonBody::new());
        let mut_body = inner.body.as_mut().unwrap();
        mut_body.insert("model".to_string(), serde_json::to_value(model).unwrap());
        mut_body.insert(
            "messages".to_string(),
            serde_json::to_value(messages).unwrap(),
        );
        ChatParam { inner }
    }

    /// 频率惩罚。一个介于-2.0和2.0之间的数值。正值根据文本中现有频率对新令牌进行惩罚，
    /// 降低模型逐字重复同一行的可能性。
    pub fn frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "frequency_penalty".to_string(),
            serde_json::to_value(frequency_penalty).unwrap(),
        );
        self
    }

    /// Logit偏置。修改指定令牌在补全中出现的可能性。
    ///
    /// 接受一个JSON对象，该对象将令牌（由分词器中的令牌ID指定）
    /// 映射到从-100到100的相关偏置值。在数学上，偏置值会在采样前添加到模型生成的logits中。
    pub fn logit_bias(mut self, logit_bias: HashMap<String, i32>) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "logit_bias".to_string(),
            serde_json::to_value(logit_bias).unwrap(),
        );
        self
    }

    /// 对数概率。是否返回输出令牌的对数概率。
    ///
    /// 如果为true，则返回`message`的`content`中每个输出令牌的对数概率。
    pub fn logprobs(mut self, logprobs: bool) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "logprobs".to_string(),
            serde_json::to_value(logprobs).unwrap(),
        );
        self
    }

    /// 输出模态。您希望模型生成的输出类型。
    ///
    /// 大多数模型都能够生成文本，这是默认值：`["text"]`。
    /// `gpt-4o-audio-preview`模型还可以生成音频。要同时请求
    /// 文本和音频响应，请使用：`["text", "audio"]`。
    pub fn modalities(mut self, modalities: Vec<Modality>) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "modalities".to_string(),
            serde_json::to_value(modalities).unwrap(),
        );
        self
    }

    /// 最大完成令牌数。补全可生成的令牌数量的上限，
    ///
    /// 包括可见输出令牌和推理令牌。
    pub fn max_completion_tokens(mut self, max_completion_tokens: i32) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "max_completion_tokens".to_string(),
            serde_json::to_value(max_completion_tokens).unwrap(),
        );
        self
    }

    /// 元数据。可附加到对象的最多16个键值对集合。
    ///
    /// 这对于以结构化格式存储有关对象的附加信息很有用。
    /// 键的最大长度为64个字符，值的最大长度为512个字符。
    pub fn metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "metadata".to_string(),
            serde_json::to_value(metadata).unwrap(),
        );
        self
    }

    /// 并行工具调用。是否在工具使用期间启用并行函数调用。
    pub fn parallel_tool_calls(mut self, parallel_tool_calls: bool) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "parallel_tool_calls".to_string(),
            serde_json::to_value(parallel_tool_calls).unwrap(),
        );
        self
    }

    /// 存在惩罚。一个介于-2.0和2.0之间的数值。正值根据新令牌是否出现在迄今为止的文本中进行惩罚，
    /// 增加模型谈论新话题的可能性。
    pub fn presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "presence_penalty".to_string(),
            serde_json::to_value(presence_penalty).unwrap(),
        );
        self
    }

    /// 生成选项数。为每个输入消息生成多少个聊天补全选项。
    ///
    /// 请注意，将根据所有选项生成的令牌总数向您收费。
    /// 将`n`保持在`1`以最小化成本。
    pub fn n(mut self, n: i32) -> Self {
        self.inner
            .body
            .as_mut()
            .unwrap()
            .insert("n".to_string(), serde_json::to_value(n).unwrap());
        self
    }

    /// 核采样参数。一种称为核采样的温度采样替代方法。
    ///
    /// 模型会考虑具有top_p概率质量的令牌结果。
    /// 因此0.1意味着只考虑构成前10%概率质量的令牌。
    /// 我们通常建议修改此参数或`temperature`，但不建议同时修改两者。
    pub fn top_p(mut self, top_p: f32) -> Self {
        self.inner
            .body
            .as_mut()
            .unwrap()
            .insert("top_p".to_string(), serde_json::to_value(top_p).unwrap());
        self
    }

    /// 采样温度。使用什么采样温度，范围在0到2之间。
    ///
    /// 较高的值如0.8会使输出更加随机，而较低的值如0.2
    /// 会使输出更加集中和确定。我们通常建议修改此参数或`top_p`，
    /// 但不建议同时修改两者。
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "temperature".to_string(),
            serde_json::to_value(temperature).unwrap(),
        );
        self
    }

    /// 终端用户标识符。代表您的终端用户的唯一标识符，这可以帮助OpenAI
    /// 监控和检测滥用行为。
    pub fn user(mut self, user: String) -> Self {
        self.inner
            .body
            .as_mut()
            .unwrap()
            .insert("user".to_string(), serde_json::to_value(user).unwrap());
        self
    }

    /// 最可能令牌数。一个介于0和20之间的整数，指定在每个令牌位置返回的最可能令牌的数量，
    /// 每个令牌都有相关的对数概率。
    /// 如果使用此参数，`logprobs`必须设置为`true`。
    pub fn top_logprobs(mut self, top_logprobs: i32) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "top_logprobs".to_string(),
            serde_json::to_value(top_logprobs).unwrap(),
        );
        self
    }

    /// 预测内容。静态预测输出内容，例如正在重新生成的文本文件的内容。
    pub fn prediction(mut self, prediction: ChatCompletionPredictionContentParam) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "prediction".to_string(),
            serde_json::to_value(prediction).unwrap(),
        );
        self
    }

    /// 推理努力程度。**仅o系列模型** - 限制推理模型的推理工作负载。
    ///
    /// 当前支持的值为`low`、`medium`和`high`。减少推理工作负载
    /// 可以加快响应时间并减少响应中用于推理的令牌数量。
    pub fn reasoning_effort(mut self, reasoning_effort: ReasoningEffort) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "reasoning_effort".to_string(),
            serde_json::to_value(reasoning_effort).unwrap(),
        );
        self
    }

    /// 服务等级。指定用于处理请求的延迟级别。
    ///
    /// 此参数与订阅了扩展级别服务的客户相关。
    /// - 如果设置为'auto'且项目启用了扩展级别，则系统将
    ///   使用扩展级别积分直到积分用完。
    /// - 如果设置为'default'，请求将使用默认服务
    ///   级别处理，该级别具有较低的正常运行时间SLA且不保证延迟。
    pub fn service_tier(mut self, service_tier: ServiceTier) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "service_tier".to_string(),
            serde_json::to_value(service_tier).unwrap(),
        );
        self
    }

    /// 工具列表。模型可能调用的工具列表。目前，仅支持函数作为工具。
    ///
    /// 使用此参数提供模型可能为其生成JSON输入的函数列表。
    /// 最多支持128个函数。
    pub fn tools(mut self, tools: Vec<ChatCompletionToolParam>) -> Self {
        self.inner
            .body
            .as_mut()
            .unwrap()
            .insert("tools".to_string(), serde_json::to_value(tools).unwrap());
        self
    }

    /// 工具选择。控制模型调用哪个（如果有）工具。
    ///
    /// - `none`表示模型不会调用任何工具，而是生成消息。
    /// - `auto`表示模型可以在生成消息或调用一个或多个工具之间进行选择。
    /// - `required`表示模型必须调用一个或多个工具。
    /// - 指定特定工具会强制模型调用该工具。
    ///
    /// 当没有工具时，默认为`none`。如果存在工具，则默认为`auto`。
    pub fn tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "tool_choice".to_string(),
            serde_json::to_value(tool_choice).unwrap(),
        );
        self
    }

    /// 重试次数。HTTP请求重试次数，覆盖客户端的全局设置。
    ///
    /// 此字段不会在请求体中序列化。
    pub fn retry_count(mut self, retry_count: usize) -> Self {
        self.inner.extensions.insert(RetryCount(retry_count));
        self
    }

    /// 超时时间。HTTP请求超时时间，覆盖客户端的全局设置。
    ///
    /// 此字段不会在请求体中序列化。
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.inner.extensions.insert(Timeout(timeout));
        self
    }

    /// 用户代理。HTTP请求User-Agent，覆盖客户端的全局设置。
    pub fn user_agent(mut self, user_agent: HeaderValue) -> Self {
        self.inner.headers.insert(USER_AGENT, user_agent);
        self
    }

    /// 设置HTTP请求头信息。
    pub fn header<K: IntoHeaderName>(mut self, key: K, val: HeaderValue) -> Self {
        self.inner.headers.insert(key, val);
        self
    }

    /// 向请求体添加额外的JSON属性。
    pub fn body<K: Into<String>, V: Into<Value>>(mut self, key: K, val: V) -> Self {
        self.inner
            .body
            .as_mut()
            .unwrap()
            .insert(key.into(), val.into());
        self
    }
}

impl ChatParam {
    pub(crate) fn take(self) -> InParam {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_request_params_serialize_with_schema() {
        let messages = vec![system!("system message"), user!(content:"user message")];

        let tool_params = Parameters::object()
            .property(
                "name",
                Parameters::string()
                    .description("name of the person")
                    .build(),
            )
            .require("name")
            .build()
            .unwrap();

        let function_tool =
            ChatCompletionToolParam::function("function_name", "function description", tool_params);

        let request = ChatParam::new("meta-llama/llama-3.3-8b-instruct:free", &messages)
            .temperature(0.1)
            .top_logprobs(1)
            .n(1)
            .tool_choice(ToolChoice::Auto)
            .tools(vec![function_tool]);

        let inner = request.take();
        let left = serde_json::to_value(&inner.body).unwrap();
        let right: serde_json::Value = serde_json::json!({
            "messages": [
                {
                    "content": "system message",
                    "role": "system"
                },
                {
                    "content": "user message",
                    "role": "user"
                }
            ],
            "model": "meta-llama/llama-3.3-8b-instruct:free",
            "n": 1,
            "temperature": 0.1,
            "tool_choice": "auto",
            "tools": [
                {
                    "function": {
                        "description": "function description",
                        "name": "function_name",
                        "parameters": {
                            "properties": {
                                "name": {
                                    "description": "name of the person",
                                    "type": "string"
                                }
                            },
                            "required": [
                                "name"
                            ],
                            "type": "object"
                        }
                    },
                    "type": "function"
                }
            ],
            "top_logprobs": 1
        });

        let left_map = left.as_object().unwrap();
        let right_map = right.as_object().unwrap();

        assert_eq!(left_map.get("messages"), right_map.get("messages"));
        assert_eq!(left_map.get("model"), right_map.get("model"));
        assert_eq!(left_map.get("n"), right_map.get("n"));
        assert_eq!(left_map.get("tool_choice"), right_map.get("tool_choice"));
        assert_eq!(left_map.get("tools"), right_map.get("tools"));
        assert_eq!(left_map.get("top_logprobs"), right_map.get("top_logprobs"));

        // Compare floating point numbers with a tolerance
        let temp_left = left_map.get("temperature").unwrap().as_f64().unwrap();
        let temp_right = right_map.get("temperature").unwrap().as_f64().unwrap();
        assert!((temp_left - temp_right).abs() < 1e-8);
    }
}
