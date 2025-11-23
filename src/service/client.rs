use crate::Config;
use crate::service::innerhttp::InnerHttp;
use std::ops::Deref;
use std::sync::Arc;

/// 一个管理底层HTTP服务和配置的高级HTTP客户端。
///
/// 此客户端充当向OpenAI API发出HTTP请求的主要入口点。
/// 它持有对`InnerHttp`的引用，该引用处理实际的请求执行，
/// 重试逻辑和配置管理。
///
/// 客户端设计为可以高效克隆，允许多个组件共享
/// 相同的底层传输层。
pub(crate) struct HttpClient {
    inner: Arc<InnerHttp>,
}

impl HttpClient {
    pub fn new(config: Config) -> HttpClient {
        HttpClient {
            inner: Arc::new(InnerHttp::new(config)),
        }
    }
}

impl Clone for HttpClient {
    fn clone(&self) -> Self {
        HttpClient {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Deref for HttpClient {
    type Target = Arc<InnerHttp>;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
