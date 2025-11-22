用于创建completions端点的参数构建器

## 必填参数

* `model` (`&str`): 用于生成补全的模型 ID
* `prompt` (`&str`): 用于生成补全的提示文本

## 可选参数（由builder模式构建）

### 生成控制参数

* `max_tokens` (`i32`): 补全中要生成的最大令牌数。提示中的令牌数加上`max_tokens`不能超过模型的上下文长度。
* `temperature` (`f32`): 使用什么采样温度，范围在0到2之间。较高的值如0.8会使输出更加随机，而较低的值如0.2会使输出更加集中和确定。我们通常建议修改此参数或`top_p`，但不建议同时修改两者。
* `top_p` (`f32`): 一种称为核采样的温度采样替代方法，其中模型考虑具有top_p概率质量的令牌结果。因此0.1意味着只考虑构成前10%概率质量的令牌。我们通常建议修改此参数或`temperature`，但不建议同时修改两者。
* `n` (`i32`): 为每个提示生成多少个补全。请注意，将根据所有补全中生成的令牌总数向您收费。将`n`保持在`1`以最小化成本。
* `best_of` (`i32`): 在服务器端生成`best_of`个补全并返回"最佳"（每个令牌具有最高对数概率的那个）。结果无法流式传输。与`n`一起使用时，`best_of`控制候选补全的数量，而`n`指定返回多少个。`best_of`必须大于或等于`n`。

### 概率和采样参数

* `logprobs` (`i32`): 在`logprobs`最可能的令牌上包含对数概率。设置为0以禁用返回任何对数概率。
* `logit_bias` (`HashMap<String, i32>`): 修改指定令牌在补全中出现的可能性。接受一个JSON对象，该对象将令牌（由分词器中的令牌ID指定）映射到从-100到100的相关偏置值。在数学上，偏置值会在采样前添加到模型生成的logits中。

### 文本处理参数

* `echo` (`bool`): 除了补全外，还回显提示。这对于调试和理解模型的行为很有用。
* `stop` (`Vec<String>`): 最多4个序列，API将在这些序列处停止生成更多令牌。返回的文本将不包含停止序列。
* `presence_penalty` (`f32`): 一个介于-2.0和2.0之间的数值。正值根据新令牌是否出现在迄今为止的文本中进行惩罚，增加模型谈论新话题的可能性。
* `frequency_penalty` (`f32`): 一个介于-2.0和2.0之间的数值。正值根据新令牌在迄今为止文本中的现有频率进行惩罚，降低模型逐字重复同一行的可能性。

### 服务和配置参数

* `user` (`String`): 代表您的终端用户的唯一标识符，这可以帮助OpenAI监控和检测滥用行为。
* `retry_count` (`usize`): HTTP请求重试次数，覆盖客户端的全局设置。此字段不会在请求体中序列化。
* `timeout` (`Duration`): HTTP请求超时时间，覆盖客户端的全局设置。此字段不会在请求体中序列化。
* `user_agent` (`HeaderValue`): HTTP请求User-Agent，覆盖客户端的全局设置。此字段不会在请求体中序列化。
* `header` (`K: IntoHeaderName, HeaderValue`): 随请求发送额外的头信息。
* `body` (`K: Into<String>, V: Into<Value>`): 向请求添加额外的JSON属性。此字段不会在请求体中序列化。

## example

```rust
use openai4rs::CompletionsParam;
let model = "gpt-3.5-turbo-instruct";
let prompt = "Once upon a time";
let params = CompletionsParam::new(model, prompt)
    .temperature(0.7)
    .max_tokens(100)
    .n(1);
```