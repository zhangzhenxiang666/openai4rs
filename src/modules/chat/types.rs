use crate::chat::tool_parameters::Parameters;
use crate::common::types::{CompletionGeneric, try_deserialize_or_skip};
use crate::content;
use crate::utils::methods::merge_extra_fields_in_place;
use derive_builder::Builder;
use serde::de::{self, MapAccess, Visitor};
use serde::ser::SerializeStruct;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt;

pub type ChatCompletion = CompletionGeneric<FinalChoice>;
pub type ChatCompletionChunk = CompletionGeneric<StreamChoice>;

#[derive(Debug, Clone, Deserialize)]
pub struct FinalChoice {
    pub index: usize,
    pub finish_reason: FinishReason,
    pub message: ChatCompletionMessage,
    pub logprobs: Option<ChoiceLogprobs>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamChoice {
    pub index: usize,
    pub delta: ChoiceDelta,
    pub finish_reason: Option<FinishReason>,
    pub logprobs: Option<ChoiceLogprobs>,
}

#[derive(Debug, Clone)]
pub struct ChoiceDelta {
    pub content: Option<String>,
    pub refusal: Option<String>,
    pub reasoning: Option<String>,
    pub role: Option<String>,
    pub tool_calls: Option<Vec<ChatCompletionToolCall>>,
    pub extra_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoice {
    Auto,
    None,
    Required,
}

#[derive(Debug, Clone)]
pub struct ChatCompletionMessage {
    pub role: String,
    pub content: Option<String>,
    pub refusal: Option<String>,
    pub reasoning: Option<String>,
    pub annotations: Option<Vec<Annotation>>,
    pub tool_calls: Option<Vec<ChatCompletionToolCall>>,
    pub extra_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone)]
pub struct ChatCompletionToolCall {
    pub index: usize,
    pub function: Function,
    pub r#type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Annotation {
    pub r#type: String,
    pub url_citation: AnnotationURLCitation,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AnnotationURLCitation {
    pub end_index: i64,
    pub start_index: i64,
    pub title: String,
    pub url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChoiceLogprobs {
    pub content: Option<Vec<ChatCompletionTokenLogprob>>,
    pub refusal: Option<Vec<ChatCompletionTokenLogprob>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionTokenLogprob {
    pub logprob: f64,
    pub token: String,
    pub bytes: Option<Vec<u8>>,
    pub top_logprobs: Option<Vec<TopLogprob>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopLogprob {
    pub logprob: f64,
    pub token: String,
    pub bytes: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    FunctionCall,
}

#[derive(Debug, Clone)]
pub enum ChatCompletionMessageParam {
    System(ChatCompletionSystemMessageParam),
    User(ChatCompletionUserMessageParam),
    Assistant(ChatCompletionAssistantMessageParam),
    Tool(ChatCompletionToolMessageParam),
    // TODO 实现 Developer
    // Developer,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionSystemMessageParam {
    pub content: Content,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionAssistantMessageParam {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ChatCompletionMessageToolCallParam>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionUserMessageParam {
    pub content: Content,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChatCompletionToolMessageParam {
    pub tool_call_id: String,
    pub content: Content,
}

#[derive(Debug, Clone)]
pub enum ChatCompletionToolParam {
    Function(FunctionDefinition),
}

#[derive(Debug, Clone, Serialize, Deserialize, Builder)]
#[builder(
    name = "FunctionDefinitionBuilder",
    pattern = "owned",
    setter(strip_option = true)
)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Parameters,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[builder(default = None)]
    pub strict: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Content {
    Text(String),
    Object(serde_json::Value),
}

#[derive(Debug, Clone)]
pub struct Function {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone)]
pub enum ChatCompletionMessageToolCallParam {
    Function(Function),
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Modality {
    Text,
    Audio,
}

#[derive(Debug, Clone)]
pub struct ChatCompletionPredictionContentParam {
    pub content: Content,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

impl ChatCompletion {
    /// 检查第一个选择的消息是否包含任何内容。
    pub fn has_content(&self) -> bool {
        self.choices
            .first()
            .map(|choice| choice.message.has_content())
            .unwrap_or(false)
    }

    /// 返回第一个选择的消息的文本内容（如果可用）。
    /// 这是访问模型响应的最常见方式。
    pub fn content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|choice| choice.message.content())
    }
    /// 检查第一个选择的消息是否包含任何工具调用。
    pub fn has_tool_calls(&self) -> bool {
        self.choices
            .first()
            .map(|choice| choice.message.has_tool_calls())
            .unwrap_or(false)
    }

    /// 返回第一个选择的消息中工具调用列表的引用（如果有的话）。
    pub fn tool_calls(&self) -> Option<&Vec<ChatCompletionToolCall>> {
        self.choices
            .first()
            .and_then(|choice| choice.message.tool_calls())
    }

    /// 检查第一个选择消息是否包含任何推理。
    pub fn has_reasoning(&self) -> bool {
        self.choices
            .first()
            .map(|choice| choice.message.has_reasoning())
            .unwrap_or(false)
    }

    /// 获取第一个选择消息的推理（如果可用）。
    pub fn reasoning(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|choice| choice.message.reasoning())
    }

    /// 返回第一个选择的消息对象的引用。
    /// 当您需要访问消息的其他属性时（如 `role` 或 `refusal`），这很有用。
    pub fn first_choice_message(&self) -> Option<&ChatCompletionMessage> {
        self.choices.first().map(|choice| &choice.message)
    }
}

impl ChatCompletionChunk {
    /// 检查第一个选择的增量是否包含任何内容。
    pub fn has_content(&self) -> bool {
        self.choices
            .first()
            .map(|choice| choice.delta.has_content())
            .unwrap_or(false)
    }

    /// 返回第一个选择的增量中的文本内容（如果可用）。
    /// 这是访问流式内容块的便捷方式。
    pub fn content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|choice| choice.delta.content())
    }

    /// 检查第一个选择的增量是否包含任何工具调用。
    pub fn has_tool_calls(&self) -> bool {
        self.choices
            .first()
            .map(|choice| choice.delta.has_tool_calls())
            .unwrap_or(false)
    }

    /// 返回第一个选择的增量中工具调用列表的引用（如果有的话）。
    pub fn tool_calls(&self) -> Option<&Vec<ChatCompletionToolCall>> {
        self.choices
            .first()
            .and_then(|choice| choice.delta.tool_calls())
    }

    /// 检查第一个选择的增量是否包含推理内容。
    pub fn has_reasoning(&self) -> bool {
        self.choices
            .first()
            .map(|choice| choice.delta.has_reasoning())
            .unwrap_or(false)
    }

    /// 返回第一个选择的增量中的推理内容（如果可用）。
    pub fn reasoning(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|choice| choice.delta.reasoning())
    }

    /// 返回块中所有选择增量的迭代器。
    pub fn deltas(&self) -> impl Iterator<Item = &ChoiceDelta> {
        self.choices.iter().map(|choice| &choice.delta)
    }
}

impl ChatCompletionMessage {
    pub fn has_tool_calls(&self) -> bool {
        self.tool_calls
            .as_ref()
            .is_some_and(|calls| !calls.is_empty())
    }
    pub fn has_reasoning(&self) -> bool {
        self.reasoning.as_ref().is_some_and(|reas| !reas.is_empty())
    }

    pub fn has_content(&self) -> bool {
        self.content.as_ref().is_some_and(|c| !c.is_empty())
    }

    pub fn content(&self) -> Option<&str> {
        self.content.as_deref()
    }

    pub fn reasoning(&self) -> Option<&str> {
        self.reasoning.as_deref()
    }

    pub fn tool_calls(&self) -> Option<&Vec<ChatCompletionToolCall>> {
        self.tool_calls.as_ref()
    }
}

impl ChoiceDelta {
    pub fn has_tool_calls(&self) -> bool {
        self.tool_calls
            .as_ref()
            .is_some_and(|calls| !calls.is_empty())
    }

    pub fn has_reasoning(&self) -> bool {
        self.reasoning.as_ref().is_some_and(|reas| !reas.is_empty())
    }

    pub fn has_content(&self) -> bool {
        self.content.as_ref().is_some_and(|c| !c.is_empty())
    }
    pub fn content(&self) -> Option<&str> {
        self.content.as_deref()
    }

    pub fn reasoning(&self) -> Option<&str> {
        self.reasoning.as_deref()
    }

    pub fn tool_calls(&self) -> Option<&Vec<ChatCompletionToolCall>> {
        self.tool_calls.as_ref()
    }
}

impl FunctionDefinition {
    /// 创建一个新的 `FunctionDefinitionBuilder` 来构建 `FunctionDefinition`。
    pub fn builder() -> FunctionDefinitionBuilder {
        FunctionDefinitionBuilder::create_empty()
    }
}

impl ChatCompletionToolParam {
    /// 使用类型安全的 `Parameters` 创建新的函数工具参数。
    pub fn function(name: &str, description: &str, parameters: Parameters) -> Self {
        Self::Function(
            FunctionDefinition::builder()
                .name(name.to_string())
                .description(description.to_string())
                .parameters(parameters)
                .build()
                .unwrap(), // Safe to unwrap as all required fields are provided
        )
    }
}

impl Function {
    pub fn new(id: &str, name: &str, arguments: &str) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            arguments: arguments.into(),
        }
    }
}

impl ChatCompletionMessageToolCallParam {
    pub fn function(id: &str, name: &str, arguments: &str) -> Self {
        Self::Function(Function::new(id, name, arguments))
    }
}

impl From<ChatCompletionToolCall> for ChatCompletionMessageToolCallParam {
    fn from(value: ChatCompletionToolCall) -> Self {
        Self::Function(value.function)
    }
}

impl From<ChatCompletionMessage> for ChatCompletionMessageParam {
    fn from(value: ChatCompletionMessage) -> Self {
        Self::Assistant(ChatCompletionAssistantMessageParam {
            name: None,
            content: value.content.map(|content| content!(content)),
            refusal: value.refusal,
            tool_calls: value.tool_calls.map(|tool_calls| {
                tool_calls
                    .into_iter()
                    .map(|tool_call| tool_call.into())
                    .collect()
            }),
        })
    }
}

impl From<ChoiceDelta> for ChatCompletionMessageParam {
    fn from(value: ChoiceDelta) -> Self {
        Self::Assistant(ChatCompletionAssistantMessageParam {
            name: None,
            content: value.content.map(|content| content!(content)),
            refusal: value.refusal,
            tool_calls: value.tool_calls.map(|tool_calls| {
                tool_calls
                    .into_iter()
                    .map(|tool_call| tool_call.into())
                    .collect()
            }),
        })
    }
}

impl From<ChoiceDelta> for ChatCompletionMessage {
    fn from(value: ChoiceDelta) -> Self {
        Self {
            content: value.content,
            refusal: value.refusal,
            role: value.role.unwrap_or("assistant".into()),
            annotations: None,
            tool_calls: value.tool_calls,
            reasoning: value.reasoning,
            extra_fields: value.extra_fields,
        }
    }
}

impl From<StreamChoice> for FinalChoice {
    fn from(value: StreamChoice) -> Self {
        Self {
            index: value.index,
            finish_reason: value.finish_reason.unwrap_or(FinishReason::Stop),
            logprobs: value.logprobs,
            message: value.delta.into(),
        }
    }
}

impl StreamChoice {
    pub fn merge(&mut self, delta: Self) {
        if self.index == 0 {
            self.index = delta.index;
        }
        if delta.finish_reason.is_some() {
            self.finish_reason = delta.finish_reason;
        }
        if delta.logprobs.is_some() {
            self.logprobs = delta.logprobs;
        }
        self.delta.merge(delta.delta);
    }
}

impl ChoiceDelta {
    pub fn merge(&mut self, delta: Self) {
        // 合并响应内容
        match (self.content.as_mut(), delta.content) {
            (Some(left), Some(right)) => left.push_str(&right),
            (None, Some(right)) => self.content = Some(right),
            _ => {}
        }

        // 如果增量中存在拒绝内容则更新
        if delta.refusal.is_some() {
            self.refusal = delta.refusal;
        }

        // 如果增量中存在角色则更新
        if delta.role.is_some() {
            self.role = delta.role;
        }

        // 使用自适应逻辑合并工具调用
        match (self.tool_calls.as_mut(), delta.tool_calls) {
            (Some(left), Some(right)) => {
                // 检测非标准、顺序的工具调用流的启发式方法。
                // 如果传入的增量有一个索引为0的工具调用，
                // 我们假设它是 `left` 向量中最后一个工具调用的延续。
                if right.len() == 1 && right[0].index == 0 {
                    if let Some(last_tool_call) = left.last_mut() {
                        // 这是安全的，因为我们已检查 right.len() == 1。
                        if let Some(r) = right.into_iter().next() {
                            last_tool_call.merge(r);
                        }
                    } else {
                        // 如果 `left` 为空，则直接获取 `right`。
                        *left = right;
                    }
                } else {
                    // 标准的基于索引的合并，用于稳健处理并发工具调用。
                    for r in right.into_iter() {
                        if let Some(l) = left.iter_mut().find(|l| l.index == r.index) {
                            l.merge(r);
                        } else {
                            left.push(r);
                        }
                    }
                }
            }
            (None, Some(right)) => self.tool_calls = Some(right),
            _ => {}
        }

        // 合并思考内容
        match (self.reasoning.as_mut(), delta.reasoning) {
            (Some(left), Some(right)) => left.push_str(&right),
            (None, Some(right)) => self.reasoning = Some(right),
            _ => {}
        }

        // 原地合并额外字段以避免不必要的克隆
        merge_extra_fields_in_place(&mut self.extra_fields, delta.extra_fields);
    }
}

impl ChatCompletionToolCall {
    pub fn merge(&mut self, delta: Self) {
        self.index = delta.index;
        self.function.merge(delta.function);
    }
}

impl Function {
    pub fn merge(&mut self, delta: Self) {
        self.id.push_str(&delta.id);
        self.name.push_str(&delta.name);
        self.arguments.push_str(&delta.arguments);
    }
}

impl Serialize for ChatCompletionPredictionContentParam {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("ChatCompletionPredictionContentParam", 2)?;
        state.serialize_field("type", "content")?;
        state.serialize_field("content", &self.content)?;
        state.end()
    }
}

impl Serialize for Function {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("Function", 2)?;
        state.serialize_field("name", &self.name)?;
        state.serialize_field("arguments", &self.arguments)?;
        state.end()
    }
}

impl Serialize for ChatCompletionMessageToolCallParam {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Function(inner) => {
                let mut state =
                    serializer.serialize_struct("ChatCompletionMessageToolCallParam", 3)?;
                state.serialize_field("type", "function")?;
                state.serialize_field("id", &inner.id)?;
                state.serialize_field("function", inner)?;
                state.end()
            }
        }
    }
}

impl Serialize for ChatCompletionMessageParam {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::System(inner) => {
                let mut len = 2;
                if inner.name.is_some() {
                    len += 1;
                }
                let mut state = serializer.serialize_struct("ChatCompletionMessageParam", len)?;
                state.serialize_field("role", "system")?;
                state.serialize_field("content", &inner.content)?;
                if let Some(name) = &inner.name {
                    state.serialize_field("name", name)?;
                }
                state.end()
            }
            Self::User(inner) => {
                let mut len = 2;
                if inner.name.is_some() {
                    len += 1;
                }
                let mut state = serializer.serialize_struct("ChatCompletionMessageParam", len)?;
                state.serialize_field("role", "user")?;
                state.serialize_field("content", &inner.content)?;
                if let Some(name) = &inner.name {
                    state.serialize_field("name", name)?;
                }
                state.end()
            }
            Self::Assistant(inner) => {
                let mut len = 1;
                if inner.content.is_some() {
                    len += 1;
                }
                if inner.name.is_some() {
                    len += 1;
                }
                if inner.refusal.is_some() {
                    len += 1;
                }
                if inner.tool_calls.is_some() {
                    len += 1;
                }
                let mut state = serializer.serialize_struct("ChatCompletionMessageParam", len)?;
                state.serialize_field("role", "assistant")?;
                if let Some(content) = &inner.content {
                    state.serialize_field("content", content)?;
                }
                if let Some(name) = &inner.name {
                    state.serialize_field("name", name)?;
                }
                if let Some(refusal) = &inner.refusal {
                    state.serialize_field("refusal", refusal)?;
                }
                if let Some(tool_calls) = &inner.tool_calls {
                    state.serialize_field("tool_calls", tool_calls)?;
                }
                state.end()
            }
            Self::Tool(inner) => {
                let mut state = serializer.serialize_struct("ChatCompletionMessageParam", 3)?;
                state.serialize_field("role", "tool")?;
                state.serialize_field("content", &inner.content)?;
                state.serialize_field("tool_call_id", &inner.tool_call_id)?;
                state.end()
            }
        }
    }
}

impl Serialize for ChatCompletionToolParam {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Function(inner) => {
                let mut state = serializer.serialize_struct("ChatCompletionToolParam", 2)?;
                state.serialize_field("type", "function")?;
                state.serialize_field("function", inner)?;
                state.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ChatCompletionToolParam {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum ToolParamHelper {
            Typed {
                r#type: String,
                function: FunctionDefinition,
            },
            Direct(FunctionDefinition),
        }

        match ToolParamHelper::deserialize(deserializer)? {
            ToolParamHelper::Typed { r#type, function } => {
                if r#type == "function" {
                    Ok(ChatCompletionToolParam::Function(function))
                } else {
                    Err(de::Error::custom(format!(
                        "Expected type 'function', found '{}'",
                        r#type
                    )))
                }
            }
            ToolParamHelper::Direct(function) => Ok(ChatCompletionToolParam::Function(function)),
        }
    }
}

impl<'de> Deserialize<'de> for Function {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct FunctionVisitor;

        impl<'de> Visitor<'de> for FunctionVisitor {
            type Value = Function;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a Function object")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Function, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id = None;
                let mut name = None;
                let mut arguments = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = try_deserialize_or_skip(&mut map)?;
                        }
                        "name" => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = try_deserialize_or_skip(&mut map)?;
                        }
                        "arguments" => {
                            if arguments.is_some() {
                                return Err(de::Error::duplicate_field("arguments"));
                            }
                            arguments = try_deserialize_or_skip(&mut map)?;
                        }
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                Ok(Function {
                    id: id.unwrap_or_default(),
                    name: name.unwrap_or_default(),
                    arguments: arguments.unwrap_or_default(),
                })
            }
        }

        deserializer.deserialize_map(FunctionVisitor)
    }
}

impl<'de> Deserialize<'de> for ChatCompletionToolCall {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ChatCompletionToolCallVisitor;

        impl<'de> Visitor<'de> for ChatCompletionToolCallVisitor {
            type Value = ChatCompletionToolCall;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a ChatCompletionToolCall object")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ChatCompletionToolCall, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id: Option<String> = None;
                let mut r#type = None;
                let mut function_data = None;
                let mut index = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = try_deserialize_or_skip(&mut map)?;
                        }
                        "type" => {
                            if r#type.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }
                            r#type = try_deserialize_or_skip(&mut map)?;
                        }
                        "function" => {
                            if function_data.is_some() {
                                return Err(de::Error::duplicate_field("function"));
                            }
                            function_data = try_deserialize_or_skip(&mut map)?;
                        }
                        "index" => {
                            if index.is_some() {
                                return Err(de::Error::duplicate_field("index"));
                            }
                            index = try_deserialize_or_skip(&mut map)?;
                        }
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                let id = id.unwrap_or_default();
                let r#type = r#type.ok_or_else(|| de::Error::missing_field("type"))?;
                let index = index.unwrap_or(0);

                let default_function_data = serde_json::json!({
                    "id": "",
                    "name": "",
                    "arguments": ""
                });

                let function_data = function_data.unwrap_or(default_function_data);

                let mut function: Function = serde_json::from_value(function_data)
                    .map_err(|e| de::Error::custom(format!("Failed to parse function: {e}")))?;

                if function.id.is_empty() {
                    function.id = id.clone();
                }

                Ok(ChatCompletionToolCall {
                    function,
                    r#type,
                    index,
                })
            }
        }

        deserializer.deserialize_map(ChatCompletionToolCallVisitor)
    }
}

impl<'de> Deserialize<'de> for ChoiceDelta {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ChoiceDeltaVisitor;

        impl<'de> Visitor<'de> for ChoiceDeltaVisitor {
            type Value = ChoiceDelta;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a ChoiceDelta object")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ChoiceDelta, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut content = None;
                let mut refusal = None;
                let mut role = None;
                let mut tool_calls = None;
                let mut reasoning = None;
                let mut reasoning_content = None;
                let mut extra_fields = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "content" => {
                            if content.is_some() {
                                return Err(de::Error::duplicate_field("content"));
                            }
                            content = Some(map.next_value()?);
                        }
                        "refusal" => {
                            if refusal.is_some() {
                                return Err(de::Error::duplicate_field("refusal"));
                            }
                            refusal = Some(map.next_value()?);
                        }
                        "role" => {
                            if role.is_some() {
                                return Err(de::Error::duplicate_field("role"));
                            }
                            role = Some(map.next_value()?);
                        }
                        "tool_calls" => {
                            if tool_calls.is_some() {
                                return Err(de::Error::duplicate_field("tool_calls"));
                            }
                            tool_calls = Some(map.next_value()?);
                        }
                        "reasoning" => {
                            if reasoning.is_some() {
                                return Err(de::Error::duplicate_field("reasoning"));
                            }
                            reasoning = Some(map.next_value()?);
                        }
                        "reasoning_content" => {
                            if reasoning_content.is_some() {
                                return Err(de::Error::duplicate_field("reasoning_content"));
                            }
                            reasoning_content = Some(map.next_value()?);
                        }
                        _ => {
                            let value = map.next_value()?;
                            extra_fields
                                .get_or_insert_with(HashMap::new)
                                .insert(key, value);
                        }
                    }
                }

                let final_reasoning = reasoning.flatten().or(reasoning_content.flatten());

                Ok(ChoiceDelta {
                    content: content.flatten(),
                    refusal: refusal.flatten(),
                    role: role.flatten(),
                    tool_calls: tool_calls.flatten(),
                    reasoning: final_reasoning,
                    extra_fields,
                })
            }
        }
        deserializer.deserialize_map(ChoiceDeltaVisitor)
    }
}

impl<'de> Deserialize<'de> for ChatCompletionMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ChatCompletionMessageVisitor;

        impl<'de> Visitor<'de> for ChatCompletionMessageVisitor {
            type Value = ChatCompletionMessage;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a ChatCompletionMessage object")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ChatCompletionMessage, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut content: Option<Option<String>> = None;
                let mut refusal: Option<Option<String>> = None;
                let mut role: Option<Option<String>> = None;
                let mut tool_calls: Option<Option<Vec<ChatCompletionToolCall>>> = None;
                let mut annotations: Option<Option<Vec<Annotation>>> = None;
                let mut reasoning: Option<Option<String>> = None;
                let mut reasoning_content: Option<Option<String>> = None;
                let mut extra_fields: Option<HashMap<String, serde_json::Value>> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "content" => {
                            if content.is_some() {
                                return Err(de::Error::duplicate_field("content"));
                            }
                            content = Some(map.next_value()?);
                        }
                        "refusal" => {
                            if refusal.is_some() {
                                return Err(de::Error::duplicate_field("refusal"));
                            }
                            refusal = Some(map.next_value()?);
                        }
                        "role" => {
                            if role.is_some() {
                                return Err(de::Error::duplicate_field("role"));
                            }
                            role = Some(map.next_value()?);
                        }
                        "tool_calls" => {
                            if tool_calls.is_some() {
                                return Err(de::Error::duplicate_field("tool_calls"));
                            }
                            tool_calls = Some(map.next_value()?);
                        }
                        "annotations" => {
                            if annotations.is_some() {
                                return Err(de::Error::duplicate_field("annotations"));
                            }
                            annotations = Some(map.next_value()?);
                        }
                        "reasoning" => {
                            if reasoning.is_some() {
                                return Err(de::Error::duplicate_field("reasoning"));
                            }
                            reasoning = Some(map.next_value()?);
                        }
                        "reasoning_content" => {
                            if reasoning_content.is_some() {
                                return Err(de::Error::duplicate_field("reasoning_content"));
                            }
                            reasoning_content = Some(map.next_value()?);
                        }
                        _ => {
                            let value = map.next_value()?;
                            extra_fields
                                .get_or_insert_with(HashMap::new)
                                .insert(key, value);
                        }
                    }
                }

                let final_reasoning = reasoning.flatten().or(reasoning_content.flatten());
                let role = role.flatten().unwrap_or_else(|| "assistant".to_string());

                Ok(ChatCompletionMessage {
                    content: content.flatten(),
                    refusal: refusal.flatten(),
                    role,
                    tool_calls: tool_calls.flatten(),
                    annotations: annotations.flatten(),
                    reasoning: final_reasoning,
                    extra_fields,
                })
            }
        }
        deserializer.deserialize_map(ChatCompletionMessageVisitor)
    }
}
