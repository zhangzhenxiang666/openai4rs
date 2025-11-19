use crate::Config;
use crate::service::transport::HttpTransport;
use std::ops::Deref;
use std::sync::Arc;

/// 一个管理底层HTTP服务和配置的高级HTTP客户端。
///
/// 此客户端充当向OpenAI API发出HTTP请求的主要入口点。
/// 它持有对`Transport`的引用，该引用处理实际的请求执行，
/// 重试逻辑和配置管理。
///
/// 客户端设计为可以高效克隆，允许多个组件共享
/// 相同的底层传输层。
pub(crate) struct HttpClient {
    /// 负责执行请求的底层传输。
    ///
    /// 此传输处理实际的HTTP通信，包括请求构建，
    /// 响应处理、重试逻辑和连接管理。
    transport: Arc<HttpTransport>,
}

impl HttpClient {
    /// 使用给定配置创建新的`HttpClient`。
    ///
    /// 这将使用提供的配置初始化底层`Transport`。
    ///
    /// # 参数
    /// * `config` - OpenAI客户端的主要配置，包装在Arc<RwLock<>>
    ///
    /// # 返回
    /// 一个准备就绪的新HttpClient实例，用于发出API请求
    pub fn new(config: Config) -> HttpClient {
        HttpClient {
            transport: Arc::new(HttpTransport::new(config)),
        }
    }
}

impl Clone for HttpClient {
    /// 创建HttpClient的克隆。
    ///
    /// 此操作是高效的，因为它只克隆传输的Arc引用，
    /// 而不是传输本身。
    fn clone(&self) -> Self {
        HttpClient {
            transport: Arc::clone(&self.transport),
        }
    }
}

impl Deref for HttpClient {
    type Target = Arc<HttpTransport>;
    fn deref(&self) -> &Self::Target {
        &self.transport
    }
}
