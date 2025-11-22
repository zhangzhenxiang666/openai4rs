用于创建embeddings端点的参数构建器

## 必填参数

* `model` (`&str`): 用于生成嵌入的模型 ID
* `input` (实现`Into<Input>`的类型`&str` or `&Vec<String>`): 要嵌入的输入文本，编码为字符串或令牌数组。要在一个请求中嵌入多个输入，请传递字符串数组或令牌数组的数组。输入不得超过模型的最大输入令牌数（所有嵌入模型为8192个令牌），不能为空字符串，并且任何数组最多必须为2048维。有关计算令牌的Python代码示例。除了每个输入的令牌限制外，所有嵌入模型都强制执行单个请求中所有输入的总计300,000个令牌的最大限制。

## 可选参数（由builder模式构建）

## 嵌入配置参数

* `dimensions` (`i32`): 结果输出嵌入应具有的维度数。仅在text-embedding-3及更高版本的模型中支持。
* `encoding_format` (`&str`): 返回嵌入的格式。可以是`float`或`base64`。默认为`float`。
* `user` (`String`): 代表您的终端用户的唯一标识符，这可以帮助OpenAI监控和检测滥用行为。

## 请求配置参数

* `retry_count` (`usize`): HTTP请求重试次数，覆盖客户端的全局设置。此字段不会在请求体中序列化。
* `timeout` (`Duration`): HTTP请求超时时间，覆盖客户端的全局设置。此字段不会在请求体中序列化。
* `user_agent` (`HeaderValue`): HTTP请求User-Agent，覆盖客户端的全局设置。此字段不会在请求体中序列化。
* `header` (`K: IntoHeaderName, HeaderValue`): 随请求发送额外的头信息。
* `body` (`K: Into<String>, V: Into<Value>`): 向请求添加额外的JSON属性。此字段不会在请求体中序列化。

## example

```rust
use openai4rs::*;
let model = "text-embedding-ada-002";
let input = "Hello, world!";
let params = EmbeddingsParam::new(model, input)
    .encoding_format(EncodingFormat::Base64)
    .user("user-123");
```
