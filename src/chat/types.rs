use crate::common::types::{CompletionGeneric, extract_optional, try_deserialize_or_skip};
use crate::content;
use crate::utils::methods::merge_extra_metadata;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

pub type ChatCompletion = CompletionGeneric<UnStreamChoice>;
pub type ChatCompletionChunk = CompletionGeneric<StreamChoice>;

#[derive(Debug, Clone, Deserialize)]
pub struct UnStreamChoice {
    pub index: i64,

    pub finish_reason: FinishReason,

    pub logprobs: Option<ChoiceLogprobs>,

    pub message: ChatCompletionMessage,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamChoice {
    pub index: i64,

    pub finish_reason: Option<FinishReason>,

    pub logprobs: Option<ChoiceLogprobs>,
    pub delta: ChoiceDelta,
}

#[derive(Debug, Clone)]
pub struct ChoiceDelta {
    pub content: Option<String>,

    pub refusal: Option<String>,

    pub role: Option<String>,

    pub tool_calls: Option<Vec<ChatCompletionToolCall>>,

    pub reasoning: Option<String>,

    pub extra_metadata: Option<HashMap<String, serde_json::Value>>,
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
    pub content: Option<String>,

    pub refusal: Option<String>,

    pub role: String,

    pub annotations: Option<Vec<Annotation>>,

    pub tool_calls: Option<Vec<ChatCompletionToolCall>>,

    pub reasoning: Option<String>,

    pub extra_metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone)]

pub struct ChatCompletionToolCall {
    pub index: i64,
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
    pub token: String,
    pub bytes: Option<Vec<u8>>,
    pub logprob: f64,
    pub top_logprobs: Option<Vec<TopLogprob>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopLogprob {
    pub token: String,
    pub logprob: f64,
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
    // TODO implement Developer
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

#[derive(Debug, Clone, Serialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict: Option<bool>,
    pub parameters: serde_json::Value,
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

impl ChatCompletionMessage {
    pub fn is_tool_calls(&self) -> bool {
        self.tool_calls
            .as_ref()
            .is_some_and(|calls| !calls.is_empty())
    }
    pub fn is_reasoning(&self) -> bool {
        self.reasoning.as_ref().is_some_and(|reas| !reas.is_empty())
    }

    pub fn get_content_str(&self) -> &str {
        match self.content.as_ref() {
            Some(content) => content.as_str(),
            None => "",
        }
    }

    pub fn get_reasoning_str(&self) -> &str {
        match self.reasoning.as_ref() {
            Some(reasoning) => reasoning.as_str(),
            None => "",
        }
    }

    pub fn get_content(self) -> String {
        match self.content {
            Some(content) => content,
            None => "".into(),
        }
    }
}

impl ChoiceDelta {
    pub fn is_tool_calls(&self) -> bool {
        self.tool_calls
            .as_ref()
            .is_some_and(|calls| !calls.is_empty())
    }

    pub fn is_reasoning(&self) -> bool {
        self.reasoning.as_ref().is_some_and(|reas| !reas.is_empty())
    }

    pub fn get_content_str(&self) -> &str {
        match self.content.as_ref() {
            Some(content) => content.as_str(),
            None => "",
        }
    }

    pub fn get_reasoning_str(&self) -> &str {
        match self.reasoning.as_ref() {
            Some(reasoning) => reasoning.as_str(),
            None => "",
        }
    }

    pub fn get_content(self) -> String {
        match self.content {
            Some(content) => content,
            None => "".into(),
        }
    }
}

impl FunctionDefinition {
    pub fn new(
        name: &str,
        description: &str,
        parameters: serde_json::Value,
        strict: Option<bool>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
            strict,
        }
    }
}

impl ChatCompletionToolParam {
    pub fn function(name: &str, description: &str, parameters: serde_json::Value) -> Self {
        Self::Function(FunctionDefinition::new(name, description, parameters, None))
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

impl ChatCompletionMessageParam {
    pub fn assistant_from_str(content: &str) -> Self {
        Self::Assistant(ChatCompletionAssistantMessageParam {
            name: None,
            content: Some(content!(content)),
            refusal: None,
            tool_calls: None,
        })
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
            extra_metadata: value.extra_metadata,
        }
    }
}

impl From<StreamChoice> for UnStreamChoice {
    fn from(value: StreamChoice) -> Self {
        Self {
            index: value.index,
            finish_reason: value.finish_reason.unwrap_or(FinishReason::Stop),
            logprobs: value.logprobs,
            message: value.delta.into(),
        }
    }
}

impl std::ops::Add for StreamChoice {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        if self.index == 0 {
            self.index = rhs.index;
        }
        if rhs.finish_reason.is_some() {
            self.finish_reason = rhs.finish_reason;
        }
        if rhs.logprobs.is_some() {
            self.logprobs = rhs.logprobs;
        }
        self.delta = self.delta + rhs.delta;
        self
    }
}

impl std::ops::Add for ChoiceDelta {
    type Output = Self;

    fn add(mut self, rhs: Self) -> Self::Output {
        self.content = match (self.content, rhs.content) {
            (Some(left), Some(right)) => Some(left + &right),
            (Some(left), None) => Some(left),
            (None, Some(right)) => Some(right),
            (None, None) => None,
        };

        if rhs.refusal.is_some() {
            self.refusal = rhs.refusal;
        }

        if rhs.role.is_some() {
            self.role = rhs.role;
        }

        self.tool_calls = match (self.tool_calls, rhs.tool_calls) {
            (Some(mut left), Some(right)) => {
                left = left.into_iter().zip(right).map(|(l, r)| l + r).collect();
                Some(left)
            }
            (Some(left), None) => Some(left),
            (None, Some(right)) => Some(right),
            (None, None) => None,
        };

        self.reasoning = match (self.reasoning, rhs.reasoning) {
            (Some(left), Some(right)) => Some(left + &right),
            (Some(left), None) => Some(left),
            (None, Some(right)) => Some(right),
            (None, None) => None,
        };

        self.extra_metadata = merge_extra_metadata(self.extra_metadata, rhs.extra_metadata);

        self
    }
}

impl std::ops::Add for ChatCompletionToolCall {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self.index = rhs.index;
        self.function = self.function + rhs.function;
        self
    }
}

impl std::ops::Add for Function {
    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self.id.push_str(&rhs.id);
        self.name.push_str(&rhs.name);
        self.arguments.push_str(&rhs.arguments);
        self
    }
}

impl Serialize for ChatCompletionPredictionContentParam {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serde_json::Map::new();
        map.insert("type".into(), Value::from("content"));
        map.insert(
            "content".into(),
            serde_json::to_value(&self.content)
                .map_err(|e| serde::ser::Error::custom(e.to_string()))?,
        );
        serializer.collect_map(map)
    }
}

impl Serialize for Function {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serde_json::Map::new();
        map.insert("name".into(), Value::from(self.name.as_str()));
        map.insert("arguments".into(), Value::from(self.arguments.as_str()));
        serde_json::Value::Object(map).serialize(serializer)
    }
}

impl Serialize for ChatCompletionMessageToolCallParam {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Function(inner) => {
                let mut map = serde_json::Map::new();
                map.insert("type".into(), Value::from("function"));
                map.insert("id".into(), Value::from(inner.id.as_str()));
                map.insert(
                    "function".into(),
                    serde_json::to_value(inner)
                        .map_err(|e| serde::ser::Error::custom(e.to_string()))?,
                );
                serializer.collect_map(map)
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
                let mut map = serde_json::Map::new();
                map.insert("role".into(), Value::from("system"));
                map.insert(
                    "content".into(),
                    serde_json::to_value(&inner.content)
                        .map_err(|e| serde::ser::Error::custom(e.to_string()))?,
                );
                if let Some(name) = &inner.name {
                    map.insert("name".into(), Value::from(name.clone()));
                }
                serializer.collect_map(map)
            }
            Self::User(inner) => {
                let mut map = serde_json::Map::new();
                map.insert("role".into(), Value::from("user"));
                map.insert(
                    "content".into(),
                    serde_json::to_value(&inner.content)
                        .map_err(|e| serde::ser::Error::custom(e.to_string()))?,
                );
                if let Some(name) = &inner.name {
                    map.insert("name".into(), Value::from(name.clone()));
                }
                serializer.collect_map(map)
            }
            Self::Assistant(inner) => {
                let mut map = serde_json::Map::new();
                map.insert("role".into(), Value::from("assistant"));
                if let Some(content) = &inner.content {
                    map.insert(
                        "content".into(),
                        serde_json::to_value(content)
                            .map_err(|e| serde::ser::Error::custom(e.to_string()))?,
                    );
                }
                if let Some(name) = &inner.name {
                    map.insert("name".into(), Value::from(name.clone()));
                }
                if let Some(refusal) = &inner.refusal {
                    map.insert("refusal".into(), Value::from(refusal.clone()));
                }
                if let Some(tool_calls) = &inner.tool_calls {
                    map.insert(
                        "tool_calls".into(),
                        serde_json::to_value(tool_calls)
                            .map_err(|e| serde::ser::Error::custom(e.to_string()))?,
                    );
                }
                serializer.collect_map(map)
            }
            Self::Tool(inner) => {
                let mut map = serde_json::Map::new();
                map.insert("role".into(), Value::from("tool"));
                map.insert(
                    "content".into(),
                    serde_json::to_value(&inner.content)
                        .map_err(|e| serde::ser::Error::custom(e.to_string()))?,
                );
                map.insert(
                    "tool_call_id".into(),
                    Value::from(inner.tool_call_id.as_str()),
                );
                serializer.collect_map(map)
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
                let mut map = serde_json::Map::new();
                map.insert("type".into(), Value::from("function"));
                map.insert(
                    "function".into(),
                    serde_json::to_value(inner)
                        .map_err(|e| serde::ser::Error::custom(e.to_string()))?,
                );
                serializer.collect_map(map)
            }
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
                let mut id: Option<String> = None;
                let mut name: Option<String> = None;
                let mut arguments: Option<String> = None;

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
                            let _: serde_json::Value = map.next_value()?;
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
                let mut r#type: Option<String> = None;
                let mut function_data: Option<serde_json::Value> = None;
                let mut index: Option<i64> = None;

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
                            let _: serde_json::Value = map.next_value()?;
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
                    .map_err(|e| de::Error::custom(format!("Failed to parse function: {}", e)))?;

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
        let mut map = HashMap::<String, serde_json::Value>::deserialize(deserializer)?;

        let content = extract_optional(&mut map, "content")?;
        let refusal = extract_optional(&mut map, "refusal")?;
        let role = extract_optional(&mut map, "role")?;
        let tool_calls = extract_optional(&mut map, "tool_calls")?;

        let reasoning = match (map.remove("reasoning"), map.remove("reasoning_content")) {
            (Some(serde_json::Value::Null), Some(serde_json::Value::Null)) => None,
            (Some(value), _) if !value.is_null() => {
                Some(serde_json::from_value(value).map_err(serde::de::Error::custom)?)
            }
            (_, Some(value)) if !value.is_null() => {
                Some(serde_json::from_value(value).map_err(serde::de::Error::custom)?)
            }
            _ => None,
        };

        let extra_metadata = if map.is_empty() { None } else { Some(map) };

        Ok(ChoiceDelta {
            content,
            refusal,
            role,
            tool_calls,
            reasoning,
            extra_metadata,
        })
    }
}

impl<'de> Deserialize<'de> for ChatCompletionMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut map = HashMap::<String, serde_json::Value>::deserialize(deserializer)?;

        let content = extract_optional(&mut map, "content")?;
        let refusal = extract_optional(&mut map, "refusal")?;

        let role: Option<String> = extract_optional(&mut map, "role")?;
        let role = role.unwrap_or("assistant".into());

        let tool_calls = extract_optional(&mut map, "tool_calls")?;
        let annotations = extract_optional(&mut map, "annotations")?;

        let reasoning = match (map.remove("reasoning"), map.remove("reasoning_content")) {
            (Some(serde_json::Value::Null), Some(serde_json::Value::Null)) => None,
            (Some(value), _) if !value.is_null() => {
                Some(serde_json::from_value(value).map_err(serde::de::Error::custom)?)
            }
            (_, Some(value)) if !value.is_null() => {
                Some(serde_json::from_value(value).map_err(serde::de::Error::custom)?)
            }
            _ => None,
        };

        let extra_metadata = if map.is_empty() { None } else { Some(map) };

        Ok(ChatCompletionMessage {
            content,
            refusal,
            role,
            tool_calls,
            annotations,
            reasoning,
            extra_metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{chat_request, system, user};
    use std::fs;

    #[test]
    fn test_deserialize_chatcompletion() {
        let json = fs::read_to_string("./assets/chatcompletion.json").unwrap();
        let chatcompletion: Result<ChatCompletion, _> = serde_json::from_str(json.as_str());
        assert!(chatcompletion.is_ok());
    }

    #[test]
    fn test_deserialize_chatcompletion_stream() {
        let json = fs::read_to_string("./assets/chatcompletionchunk.json").unwrap();

        let chatcompletion_chunk: Result<ChatCompletionChunk, _> =
            serde_json::from_str(json.as_str());
        assert!(chatcompletion_chunk.is_ok());
    }

    #[test]
    fn test_assistant_serialize() {
        let assistant =
            ChatCompletionMessageParam::Assistant(ChatCompletionAssistantMessageParam {
                content: Some(content!("content")),
                name: Some("name".into()),
                refusal: Some("refusal".into()),
                tool_calls: Some(vec![ChatCompletionMessageToolCallParam::function(
                    "id",
                    "name",
                    "{'path': '/.cargo'}",
                )]),
            });
        let json = serde_json::to_string(&assistant).unwrap();
        assert_eq!(
            &json,
            r#"{"content":"content","name":"name","refusal":"refusal","role":"assistant","tool_calls":[{"function":{"arguments":"{'path': '/.cargo'}","name":"name"},"id":"id","type":"function"}]}"#
        );
    }

    #[test]
    fn test_request_params_serialize() {
        let messages = vec![system!("system message"), user!("user message")];
        let request = chat_request("meta-llama/llama-3.3-8b-instruct:free", &messages)
            .temperature(0.1)
            .top_logprobs(1)
            .n(1)
            .top_logprobs(1)
            .max_tokens(1024)
            .tool_choice(ToolChoice::Auto)
            .tools(vec![ChatCompletionToolParam::function(
                "function_name",
                "function description",
                serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "name of the person"
                        }
                    },
                    "required": ["name"]
                }),
            )])
            .build()
            .unwrap();
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(
            &json,
            r#"{"model":"meta-llama/llama-3.3-8b-instruct:free","messages":[{"content":"system message","role":"system"},{"content":"user message","role":"user"}],"max_tokens":1024,"n":1,"temperature":0.1,"top_logprobs":1,"tools":[{"function":{"description":"function description","name":"function_name","parameters":{"properties":{"name":{"description":"name of the person","type":"string"}},"required":["name"],"type":"object"}},"type":"function"}],"tool_choice":"auto"}"#
        );
    }
}
