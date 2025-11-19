use super::types::{EncodingFormat, Input};
use crate::common::types::{Body, InParam, RetryCount, Timeout};
use http::{
    HeaderValue,
    header::{IntoHeaderName, USER_AGENT},
};
use serde_json::Value;
use std::time::Duration;

pub struct EmbeddingsParam {
    inner: InParam,
}

impl EmbeddingsParam {
    /// 创建embeddings api参数构建器
    /// `model`: 模型名称
    /// `input`: 输入需要嵌入的文本
    pub fn new<T: Into<Input>>(model: &str, input: T) -> Self {
        let mut inner = InParam::new();
        inner.body = Some(Body::new());
        inner
            .body
            .as_mut()
            .unwrap()
            .insert("model".to_string(), serde_json::to_value(model).unwrap());

        inner.body.as_mut().unwrap().insert(
            "input".to_string(),
            serde_json::to_value(<T as Into<Input>>::into(input)).unwrap(),
        );
        let param = EmbeddingsParam { inner };
        param.encoding_format(EncodingFormat::Float)
    }

    /// 返回嵌入的格式。可以是`float`或`base64`
    pub fn encoding_format(mut self, encoding_format: EncodingFormat) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "encoding_format".to_string(),
            serde_json::to_value(encoding_format).unwrap(),
        );
        self
    }

    /// 结果输出嵌入应该具有的维度数。
    /// 仅在`text-embedding-3`及后续模型中支持。
    pub fn dimensions(mut self, dimensions: usize) -> Self {
        self.inner.body.as_mut().unwrap().insert(
            "dimensions".to_string(),
            serde_json::to_value(dimensions).unwrap(),
        );
        self
    }

    /// 代表您的终端用户的唯一标识符，这可以帮助OpenAI
    /// 监控和检测滥用行为。
    pub fn user(mut self, user: &str) -> Self {
        self.inner
            .body
            .as_mut()
            .unwrap()
            .insert("user".to_string(), serde_json::to_value(user).unwrap());
        self
    }

    /// HTTP请求超时时间，覆盖客户端的全局设置。
    ///
    /// 此字段不会在请求体中序列化。
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.inner.extensions.insert(Timeout(timeout));
        self
    }

    /// HTTP请求User-Agent，覆盖客户端的全局设置。
    ///
    /// 此字段不会在请求体中序列化。
    pub fn user_agent(mut self, user_agent: HeaderValue) -> Self {
        self.inner.headers.insert(USER_AGENT, user_agent);
        self
    }

    /// 随请求发送额外的头信息。
    pub fn header<K: IntoHeaderName>(mut self, key: K, val: HeaderValue) -> Self {
        self.inner.headers.insert(key, val);
        self
    }

    /// 向请求添加额外的JSON属性。
    ///
    /// 此字段不会在请求体中序列化。
    pub fn body<K: Into<String>, V: Into<Value>>(mut self, key: K, val: V) -> Self {
        self.inner
            .body
            .as_mut()
            .unwrap()
            .insert(key.into(), val.into());
        self
    }

    /// HTTP请求重试次数，覆盖客户端的全局设置。
    ///
    /// 此字段不会在请求体中序列化。
    pub fn retry_count(mut self, retry_count: usize) -> Self {
        self.inner.extensions.insert(RetryCount(retry_count));
        self
    }
}

impl EmbeddingsParam {
    pub(crate) fn take(self) -> InParam {
        self.inner
    }
}
