从环境变量创建新的OpenAI客户端。

查找以下环境变量：
- `OPENAI_API_KEY` (必需): 您的API密钥
- `OPENAI_BASE_URL` (可选): 基础URL，默认为 "https://api.openai.com/v1"
- `OPENAI_TIMEOUT` (可选): 请求超时时间（秒），默认为60
- `OPENAI_CONNECT_TIMEOUT` (可选): 连接超时时间（秒），默认为10
- `OPENAI_RETRY_COUNT` (可选): 重试次数，默认为5
- `OPENAI_PROXY` (可选): HTTP代理URL
- `OPENAI_USER_AGENT` (可选): 自定义用户代理字符串

# 错误

如果环境变量中未设置`OPENAI_API_KEY`，则返回错误。

# 示例

```bash
# 设置环境变量
export OPENAI_API_KEY="sk-your-api-key"
export OPENAI_BASE_URL="https://api.openai.com/v1"  # 可选
export OPENAI_TIMEOUT="120"  # 可选，120秒
export OPENAI_RETRY_COUNT="3"  # 可选，3次重试
```

```rust
use openai4rs::OpenAI;
use dotenvy::dotenv;
#[tokio::main]
async fn main() -> Result<(), String> {
    dotenv().ok();
    let client = OpenAI::from_env()?;

    // 客户端已准备就绪
    println!("Connected to: {}", client.base_url());
    Ok(())
}
```
