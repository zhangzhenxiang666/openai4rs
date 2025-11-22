use crate::common::types::{InParam, JsonBody, RetryCount, Timeout};
use http::{
    HeaderValue,
    header::{IntoHeaderName, USER_AGENT},
};
use serde_json::Value;
use std::{collections::HashMap, time::Duration};

pub struct CompletionsParam {
    inner: InParam,
}

impl CompletionsParam {
    #[doc = include_str!("../../docs/completions_param.md")]
    pub fn new(model: &str, prompt: &str) -> Self {
        let mut inner = InParam::new();
        inner.body = Some(JsonBody::new());
        inner
            .body
            .as_mut()
            .unwrap()
            .insert("model".to_string(), serde_json::to_value(model).unwrap());

        inner
            .body
            .as_mut()
            .unwrap()
            .insert("prompt".to_string(), serde_json::to_value(prompt).unwrap());

        CompletionsParam { inner }
    }

    /// 最大令牌数。补全中要生成的最大令牌数。
    ///
    /// 提示中的令牌数加上`max_tokens`不能超过模型的上下文长度。
    pub fn max_tokens(mut self, max_tokens: i32) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "max_tokens".to_string(),
            serde_json::to_value(max_tokens).unwrap(),
        );
        self
    }

    /// 采样温度。使用什么采样温度，范围在0到2之间。
    ///
    /// 较高的值（如0.8）会使输出更加随机，而较低的值（如0.2）
    /// 会使输出更加集中和确定。
    /// 我们通常建议修改此参数或`top_p`，但不建议同时修改两者。
    pub fn temperature(mut self, temperature: f32) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "temperature".to_string(),
            serde_json::to_value(temperature).unwrap(),
        );
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

    /// 生成选项数。为每个提示生成多少个补全。
    ///
    /// 请注意，将根据所有补全中生成的令牌总数向您收费。
    /// 将`n`保持在`1`以最小化成本。
    pub fn n(mut self, n: i32) -> Self {
        self.inner
            .body
            .as_mut()
            .unwrap()
            .insert("n".to_string(), serde_json::to_value(n).unwrap());
        self
    }

    /// 对数概率。在`logprobs`最可能的令牌上包含对数概率。
    ///
    /// 设置为0以禁用返回任何对数概率。
    pub fn logprobs(mut self, logprobs: i32) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "logprobs".to_string(),
            serde_json::to_value(logprobs).unwrap(),
        );
        self
    }

    /// 回显提示。除了补全外，还回显提示。
    ///
    /// 这对于调试和理解模型的行为很有用。
    pub fn echo(mut self, echo: bool) -> Self {
        self.inner
            .body
            .as_mut()
            .unwrap()
            .insert("echo".to_string(), serde_json::to_value(echo).unwrap());
        self
    }

    /// 停止序列。最多4个序列，API将在这些序列处停止生成更多令牌。
    ///
    /// 返回的文本将不包含停止序列。
    pub fn stop(mut self, stop: Vec<String>) -> Self {
        self.inner
            .body
            .as_mut()
            .unwrap()
            .insert("stop".to_string(), serde_json::to_value(stop).unwrap());
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

    /// 频率惩罚。一个介于-2.0和2.0之间的数值。正值根据新令牌在迄今为止文本中的现有频率进行惩罚，
    /// 降低模型逐字重复同一行的可能性。
    pub fn frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "frequency_penalty".to_string(),
            serde_json::to_value(frequency_penalty).unwrap(),
        );
        self
    }

    /// 最佳补全数。在服务器端生成`best_of`个补全并返回"最佳"
    /// （每个令牌具有最高对数概率的那个）。
    ///
    /// 结果无法流式传输。与`n`一起使用时，`best_of`控制
    /// 候选补全的数量，而`n`指定返回多少个。
    /// `best_of`必须大于或等于`n`。
    pub fn best_of(mut self, best_of: i32) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "best_of".to_string(),
            serde_json::to_value(best_of).unwrap(),
        );
        self
    }

    /// Logit偏置。修改指定令牌在补全中出现的可能性。
    ///
    /// 接受一个JSON对象，该对象将令牌（由分词器中的令牌ID指定）
    /// 映射到-100到100之间的相关偏置值。
    pub fn logit_bias(mut self, bias: HashMap<String, i32>) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "logit_bias".to_string(),
            serde_json::to_value(bias).unwrap(),
        );
        self
    }

    /// 终端用户标识符。代表您的终端用户的唯一标识符，这可以帮助OpenAI监控和检测滥用行为。
    pub fn user(mut self, user: String) -> Self {
        self.inner
            .body
            .as_mut()
            .unwrap()
            .insert("user".to_string(), serde_json::to_value(user).unwrap());
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

    /// 重试次数。HTTP请求重试次数，覆盖客户端的全局设置。
    ///
    /// 此字段不会在请求体中序列化。
    pub fn retry_count(mut self, retry_count: usize) -> Self {
        self.inner.extensions.insert(RetryCount(retry_count));
        self
    }
}

impl CompletionsParam {
    pub(crate) fn take(self) -> InParam {
        self.inner
    }
}
