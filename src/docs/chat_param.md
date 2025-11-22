用于创建chat端点的参数构建器

## 必填参数

* `model` (`&str`): 用于生成响应的模型 ID
* `messages` (`&Vec<ChatCompletionMessageParam>`): 截至目前为止的对话消息列表。取决于您使用的模型支持不同的消息类型（模式），例如文本、图片和音频。

## 可选参数（由builder模式构建）

## 生成控制参数

* `frequency_penalty` (`f32`): 一个介于-2.0和2.0之间的数值。正值根据文本中现有频率对新令牌进行惩罚，降低模型逐字重复同一行的可能性。
* `presence_penalty` (`f32`): 一个介于-2.0和2.0之间的数值。正值根据新令牌是否出现在迄今为止的文本中进行惩罚，增加模型谈论新话题的可能性。
* `temperature` (`f32`): 使用什么采样温度，范围在0到2之间。较高的值如0.8会使输出更加随机，而较低的值如0.2会使输出更加集中和确定。我们通常建议修改此参数或`top_p`，但不建议同时修改两者。
* `top_p` (`f32`): 一种称为核采样的温度采样替代方法。模型会考虑具有top_p概率质量的令牌结果。因此0.1意味着只考虑构成前10%概率质量的令牌。我们通常建议修改此参数或`temperature`，但不建议同时修改两者。
* `n` (`i32`): 为每个输入消息生成多少个聊天补全选项。请注意，将根据所有选项生成的令牌总数向您收费。将`n`保持在`1`以最小化成本。
* `max_completion_tokens` (`i32`): 补全可生成的令牌数量的上限，包括可见输出令牌和推理令牌。

## 工具调用参数

* `tools` (`Vec<ChatCompletionToolParam>`): 模型可能调用的工具列表。目前，仅支持函数作为工具。使用此参数提供模型可能为其生成JSON输入的函数列表。最多支持128个函数。
* `tool_choice` (`ToolChoice`): 控制模型调用哪个（如果有）工具。`none`表示模型不会调用任何工具，而是生成消息。`auto`表示模型可以在生成消息或调用一个或多个工具之间进行选择。`required`表示模型必须调用一个或多个工具。指定特定工具会强制模型调用该工具。当没有工具时，默认为`none`。如果存在工具，则默认为`auto`。
* `parallel_tool_calls` (`bool`): 是否在工具使用期间启用并行函数调用。

## 概率和采样参数

* `logprobs` (`bool`): 是否返回输出令牌的对数概率。如果为true，则返回`message`的`content`中每个输出令牌的对数概率。
* `top_logprobs` (`i32`): 一个介于0和20之间的整数，指定在每个令牌位置返回的最可能令牌的数量，每个令牌都有相关的对数概率。如果使用此参数，`logprobs`必须设置为`true`。
* `logit_bias` (`HashMap<String, i32>`): 修改指定令牌在补全中出现的可能性。接受一个JSON对象，该对象将令牌（由分词器中的令牌ID指定）映射到从-100到100的相关偏置值。在数学上，偏置值会在采样前添加到模型生成的logits中。

## 模态和输出参数

* `modalities` (`Vec<Modality>`): 您希望模型生成的输出类型。大多数模型都能够生成文本，这是默认值：`["text"]`。`gpt-4o-audio-preview`模型还可以生成音频。要同时请求文本和音频响应，请使用：`["text", "audio"]`。
* `prediction` (`ChatCompletionPredictionContentParam`): 静态预测输出内容，例如正在重新生成的文本文件的内容。

## 推理控制参数

* `reasoning_effort` (`ReasoningEffort`): **仅o系列模型** - 限制推理模型的推理工作负载。当前支持的值为`low`、`medium`和`high`。减少推理工作负载可以加快响应时间并减少响应中用于推理的令牌数量。

## 服务和配置参数

* `service_tier` (`ServiceTier`): 指定用于处理请求的延迟级别。此参数与订阅了扩展级别服务的客户相关。- 如果设置为'auto'且项目启用了扩展级别，则系统将使用扩展级别积分直到积分用完。- 如果设置为'default'，请求将使用默认服务级别处理，该级别具有较低的正常运行时间SLA且不保证延迟。
* `metadata` (`HashMap<String, String>`): 可附加到对象的最多16个键值对集合。这对于以结构化格式存储有关对象的附加信息很有用。键的最大长度为64个字符，值的最大长度为512个字符。
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
let model = "gpt-3.5-turbo";
let messages = vec![];
let params = ChatParam::new(model, &messages)
    .temperature(0.9)
    .n(2)
    .max_completion_tokens(100);
```
