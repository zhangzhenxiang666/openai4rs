use crate::common::types::{InParam, JsonBody, RetryCount, Timeout};
use http::{
    HeaderValue,
    header::{IntoHeaderName, USER_AGENT},
};
use serde_json::Value;
use std::time::Duration;

pub struct ModelsParam {
    inner: InParam,
}

impl ModelsParam {
    pub fn new() -> Self {
        Self {
            inner: InParam::new(),
        }
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
            .get_or_insert_with(JsonBody::new)
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

impl ModelsParam {
    pub(crate) fn take(self) -> InParam {
        self.inner
    }
}

impl Default for ModelsParam {
    fn default() -> Self {
        Self::new()
    }
}
