use crate::chat::tool_parameters::{ConversionError, Parameters};
use crate::common::types::{CompletionGeneric, extract_optional, try_deserialize_or_skip};
use crate::content;
use crate::utils::methods::merge_extra_metadata_in_place;
use derive_builder::Builder;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;

pub type ChatCompletion = CompletionGeneric<FinalChoice>;
pub type ChatCompletionChunk = CompletionGeneric<StreamChoice>;

#[derive(Debug, Clone, Deserialize)]
pub struct FinalChoice {
    pub index: i64,
    pub finish_reason: FinishReason,
    pub message: ChatCompletionMessage,
    pub logprobs: Option<ChoiceLogprobs>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StreamChoice {
    pub index: i64,
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
    pub role: String,
    pub content: Option<String>,
    pub refusal: Option<String>,
    pub reasoning: Option<String>,
    pub annotations: Option<Vec<Annotation>>,
    pub tool_calls: Option<Vec<ChatCompletionToolCall>>,
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

#[derive(Debug, Clone, Serialize, Builder)]
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
    /// Returns the text content of the first choice's message, if available.
    /// This is the most common way to access the model's response.
    pub fn content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|choice| choice.message.content())
    }

    /// Checks if the first choice's message contains any content.
    pub fn has_content(&self) -> bool {
        self.choices
            .first()
            .map(|choice| choice.message.has_content())
            .unwrap_or(false)
    }

    /// Checks if the first choice's message contains any tool calls.
    pub fn has_tool_calls(&self) -> bool {
        self.choices
            .first()
            .map(|choice| choice.message.has_tool_calls())
            .unwrap_or(false)
    }

    /// Returns a reference to the list of tool calls from the first choice's message, if any.
    pub fn tool_calls(&self) -> Option<&Vec<ChatCompletionToolCall>> {
        self.choices
            .first()
            .and_then(|choice| choice.message.tool_calls())
    }

    /// Returns a reference to the message object of the first choice.
    /// This is useful when you need to access other properties of the message,
    /// such as `role` or `refusal`.
    pub fn first_choice_message(&self) -> Option<&ChatCompletionMessage> {
        self.choices.first().map(|choice| &choice.message)
    }
}

impl ChatCompletionChunk {
    /// Returns the text content from the delta of the first choice, if available.
    /// This is a convenient way to access the streamed content chunks.
    pub fn content(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|choice| choice.delta.content())
    }

    /// Checks if the first choice's delta contains any content.
    pub fn has_content(&self) -> bool {
        self.choices
            .first()
            .map(|choice| choice.delta.has_content())
            .unwrap_or(false)
    }

    /// Checks if the first choice's delta contains any tool calls.
    pub fn has_tool_calls(&self) -> bool {
        self.choices
            .first()
            .map(|choice| choice.delta.has_tool_calls())
            .unwrap_or(false)
    }

    /// Returns a reference to the list of tool calls from the delta of the first choice, if any.
    pub fn tool_calls(&self) -> Option<&Vec<ChatCompletionToolCall>> {
        self.choices
            .first()
            .and_then(|choice| choice.delta.tool_calls())
    }

    /// Checks if the first choice's delta contains reasoning content.
    pub fn has_reasoning(&self) -> bool {
        self.choices
            .first()
            .map(|choice| choice.delta.has_reasoning())
            .unwrap_or(false)
    }

    /// Returns the reasoning content from the delta of the first choice, if available.
    pub fn reasoning(&self) -> Option<&str> {
        self.choices
            .first()
            .and_then(|choice| choice.delta.reasoning())
    }

    /// Returns an iterator over the deltas of all choices in the chunk.
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
    /// Creates a new `FunctionDefinitionBuilder` to construct a `FunctionDefinition`.
    pub fn builder() -> FunctionDefinitionBuilder {
        FunctionDefinitionBuilder::create_empty()
    }

    /// A convenient method to create a `FunctionDefinition` from a `serde_json::Value`.
    ///
    /// This method is a wrapper around `TryFrom<Value>`.
    ///
    /// # Arguments
    ///
    /// * `value` - A `serde_json::Value` that should represent a FunctionDefinition.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the constructed `FunctionDefinition` or a `ConversionError`.
    pub fn from_value(value: Value) -> Result<Self, ConversionError> {
        Self::try_from(value)
    }
}

impl ChatCompletionToolParam {
    /// Creates a new function tool parameter with type-safe `Parameters`.
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

    /// A convenient method to create a `ChatCompletionToolParam` from a `serde_json::Value`.
    ///
    /// This method is a wrapper around `TryFrom<Value>`.
    ///
    /// # Arguments
    ///
    /// * `value` - A `serde_json::Value` that should represent a ChatCompletionToolParam.
    ///
    /// # Returns
    ///
    /// A `Result` containing either the constructed `ChatCompletionToolParam` or a `ConversionError`.
    pub fn from_value(value: Value) -> Result<Self, ConversionError> {
        Self::try_from(value)
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

impl TryFrom<Value> for FunctionDefinition {
    type Error = ConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let obj = value.as_object().ok_or_else(|| {
            ConversionError::ValueNotAnObject(format!(
                "Expected object for FunctionDefinition, got: {:?} (type: {:?})",
                value, value
            ))
        })?;

        let name = obj
            .get("name")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ConversionError::InvalidFieldValue(
                    "name".to_string(),
                    "Field 'name' is required and must be a string".to_string(),
                )
            })?
            .to_string();

        let description = obj
            .get("description")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ConversionError::InvalidFieldValue(
                    "description".to_string(),
                    "Field 'description' is required and must be a string".to_string(),
                )
            })?
            .to_string();

        let parameters_value = obj.get("parameters").ok_or_else(|| {
            ConversionError::InvalidFieldValue(
                "parameters".to_string(),
                "Field 'parameters' is required".to_string(),
            )
        })?;
        let parameters = Parameters::try_from(parameters_value.clone())?;

        // Handle optional "strict" field
        let strict = obj.get("strict").and_then(Value::as_bool);

        Ok(FunctionDefinition {
            name,
            description,
            parameters,
            strict,
        })
    }
}

impl TryFrom<Value> for ChatCompletionToolParam {
    type Error = ConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let obj = value.as_object().ok_or_else(|| {
            ConversionError::ValueNotAnObject(format!(
                "Expected object for ChatCompletionToolParam, got: {:?} (type: {:?})",
                value, value
            ))
        })?;

        // Check if it's the standard format with "type" and "function" fields
        if let Some(type_str) = obj.get("type").and_then(Value::as_str) {
            if type_str == "function" {
                if let Some(function_value) = obj.get("function") {
                    let function_def = FunctionDefinition::try_from(function_value.clone())?;
                    return Ok(ChatCompletionToolParam::Function(function_def));
                } else {
                    // "type": "function" is present but "function" field is missing
                    return Err(ConversionError::InvalidFieldValue(
                        "function".to_string(),
                        "Field 'function' is required when 'type' is 'function'".to_string(),
                    ));
                }
            } else {
                // "type" field is present but not "function"
                return Err(ConversionError::InvalidFieldValue(
                    "type".to_string(),
                    format!(
                        "Expected 'function' for 'type' field, got: {} (full object: {:?})",
                        type_str, obj
                    ),
                ));
            }
        }

        // If no "type" field, assume it's the direct FunctionDefinition format
        let function_def = FunctionDefinition::try_from(value)?;
        Ok(ChatCompletionToolParam::Function(function_def))
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
        // Merge content
        match (self.content.as_mut(), delta.content) {
            (Some(left), Some(right)) => left.push_str(&right),
            (None, Some(right)) => self.content = Some(right),
            _ => {}
        }

        // Update refusal if present in delta
        if delta.refusal.is_some() {
            self.refusal = delta.refusal;
        }

        // Update role if present in delta
        if delta.role.is_some() {
            self.role = delta.role;
        }

        // Merge tool calls with adaptive logic
        match (self.tool_calls.as_mut(), delta.tool_calls) {
            (Some(left), Some(right)) => {
                // Heuristic to detect non-standard, sequential tool call streams.
                // If the incoming delta has one tool call with index 0,
                // we assume it's a continuation of the last tool call in the `left` vector.
                if right.len() == 1 && right[0].index == 0 {
                    if let Some(last_tool_call) = left.last_mut() {
                        // This is safe because we've checked right.len() == 1.
                        if let Some(r) = right.into_iter().next() {
                            last_tool_call.merge(r);
                        }
                    } else {
                        // If `left` is empty, just take `right`.
                        *left = right;
                    }
                } else {
                    // Standard, index-based merging for robust handling of concurrent tool calls.
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

        // Merge reasoning
        match (self.reasoning.as_mut(), delta.reasoning) {
            (Some(left), Some(right)) => left.push_str(&right),
            (None, Some(right)) => self.reasoning = Some(right),
            _ => {}
        }

        // Merge extra metadata in-place to avoid unnecessary cloning
        merge_extra_metadata_in_place(&mut self.extra_metadata, delta.extra_metadata);
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
                    map.insert("name".into(), Value::from(name.as_str()));
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
                    map.insert("name".into(), Value::from(name.as_str()));
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
                    map.insert("name".into(), Value::from(name.as_str()));
                }
                if let Some(refusal) = &inner.refusal {
                    map.insert("refusal".into(), Value::from(refusal.as_str()));
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
    use crate::chat::tool_parameters::Parameters;
    use crate::{chat_request, system, user};
    use openai4rs_macro::assistant;
    use std::fs;

    #[test]
    fn test_deserialize_chatcompletion() {
        let json = fs::read_to_string("./assets/chatcompletion.json").unwrap();
        let chatcompletion: Result<ChatCompletion, _> = serde_json::from_str(json.as_str());
        assert!(chatcompletion.is_ok());
    }

    #[test]
    fn test_from_value_to_function_definition() {
        let json = fs::read_to_string("./assets/function_definition.json").unwrap();
        let value: serde_json::Value = serde_json::from_str(json.as_str()).unwrap();
        let function_definition_result = FunctionDefinition::try_from(value.clone());
        assert!(function_definition_result.is_ok());

        let function_definition = function_definition_result.unwrap();
        assert_eq!(function_definition.name, "get_current_weather");
        assert_eq!(
            function_definition.description,
            "Get the current weather in a given location"
        );
        // Check parameters structure
        match &function_definition.parameters {
            Parameters::Object(obj_params) => {
                assert_eq!(obj_params.required, vec!["location"]);
                assert!(obj_params.properties.contains_key("location"));
                assert!(obj_params.properties.contains_key("unit"));
            }
            _ => panic!("Parameters should be an object"),
        }
    }

    #[test]
    fn test_from_value_to_chat_completion_tool_param() {
        // Test standard format
        let json = fs::read_to_string("./assets/chat_completion_tool_param.json").unwrap();
        let value: serde_json::Value = serde_json::from_str(json.as_str()).unwrap();
        let chat_completion_tool_param_result = ChatCompletionToolParam::try_from(value);
        assert!(chat_completion_tool_param_result.is_ok());

        // Verify the parsed data for standard format
        let ChatCompletionToolParam::Function(function_def) =
            chat_completion_tool_param_result.unwrap();

        assert_eq!(function_def.name, "get_current_weather");
        assert_eq!(
            function_def.description,
            "Get the current weather in a given location"
        );

        // Test direct FunctionDefinition format
        let json = fs::read_to_string("./assets/function_definition.json").unwrap();
        let value: serde_json::Value = serde_json::from_str(json.as_str()).unwrap();
        let chat_completion_tool_param_result = ChatCompletionToolParam::try_from(value);
        assert!(chat_completion_tool_param_result.is_ok());

        // Verify the parsed data for direct format
        let ChatCompletionToolParam::Function(function_def) =
            chat_completion_tool_param_result.unwrap();

        assert_eq!(function_def.name, "get_current_weather");
        assert_eq!(
            function_def.description,
            "Get the current weather in a given location"
        );
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
        let assistant = assistant!(
            content = "content",
            name = "name",
            refusal = "refusal",
            tool_calls = vec![ChatCompletionMessageToolCallParam::function(
                "id",
                "name",
                "{'path': '/.cargo'}",
            )]
        );

        let json = serde_json::to_string(&assistant).unwrap();
        assert_eq!(
            &json,
            r#"{"content":"content","name":"name","refusal":"refusal","role":"assistant","tool_calls":[{"function":{"arguments":"{'path': '/.cargo'}","name":"name"},"id":"id","type":"function"}]}"#
        );
    }

    #[test]
    fn test_request_params_serialize_with_schema() {
        let messages = vec![system!("system message"), user!(content:"user message")];

        let tool_params = Parameters::object()
            .property(
                "name",
                Parameters::string()
                    .description("name of the person")
                    .build(),
            )
            .require("name")
            .build()
            .unwrap();

        let function_tool =
            ChatCompletionToolParam::function("function_name", "function description", tool_params);

        let request = chat_request("meta-llama/llama-3.3-8b-instruct:free", &messages)
            .temperature(0.1)
            .top_logprobs(1)
            .n(1)
            .max_tokens(1024)
            .tool_choice(ToolChoice::Auto)
            .tools(vec![function_tool])
            .build()
            .unwrap();

        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(
            &json,
            r#"{"model":"meta-llama/llama-3.3-8b-instruct:free","messages":[{"content":"system message","role":"system"},{"content":"user message","role":"user"}],"max_tokens":1024,"n":1,"temperature":0.1,"top_logprobs":1,"tools":[{"function":{"description":"function description","name":"function_name","parameters":{"properties":{"name":{"description":"name of the person","type":"string"}},"required":["name"],"type":"object"}},"type":"function"}],"tool_choice":"auto"}"#
        );
    }

    #[test]
    fn test_chat_completion_helpers() {
        let message = ChatCompletionMessage {
            role: "assistant".to_string(),
            content: Some("Hello, world!".to_string()),
            refusal: None,
            reasoning: None,
            annotations: None,
            tool_calls: Some(vec![ChatCompletionToolCall {
                index: 0,
                function: Function {
                    id: "call_123".to_string(),
                    name: "get_current_weather".to_string(),
                    arguments: r#"{"location": "Boston, MA"}"#.to_string(),
                },
                r#type: "function".to_string(),
            }]),
            extra_metadata: None,
        };

        let choice = FinalChoice {
            index: 0,
            finish_reason: FinishReason::Stop,
            message: message.clone(),
            logprobs: None,
        };

        let chat_completion = ChatCompletion {
            id: "chatcmpl-123".to_string(),
            choices: vec![choice],
            created: 1234567890,
            model: "gpt-3.5-turbo".to_string(),
            object: "chat.completion".to_string(),
            usage: None,
            service_tier: None,
            system_fingerprint: None,
            extra_metadata: None,
        };

        // Test content()
        assert_eq!(chat_completion.content(), Some("Hello, world!"));

        // Test has_tool_calls()
        assert!(chat_completion.has_tool_calls());

        // Test tool_calls()
        let tool_calls = chat_completion.tool_calls().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].function.name, "get_current_weather");
    }

    #[test]
    fn test_chat_completion_chunk_helpers() {
        let delta = ChoiceDelta {
            content: Some("Hello, world!".to_string()),
            refusal: None,
            reasoning: None,
            role: Some("assistant".to_string()),
            tool_calls: Some(vec![ChatCompletionToolCall {
                index: 0,
                function: Function {
                    id: "call_123".to_string(),
                    name: "get_current_weather".to_string(),
                    arguments: r#"{"location": "Boston, MA"}"#.to_string(),
                },
                r#type: "function".to_string(),
            }]),
            extra_metadata: None,
        };

        let choice = StreamChoice {
            index: 0,
            delta: delta.clone(),
            finish_reason: Some(FinishReason::Stop),
            logprobs: None,
        };

        let chat_completion_chunk = ChatCompletionChunk {
            id: "chatcmpl-123".to_string(),
            choices: vec![choice],
            created: 1234567890,
            model: "gpt-3.5-turbo".to_string(),
            object: "chat.completion.chunk".to_string(),
            usage: None,
            service_tier: None,
            system_fingerprint: None,
            extra_metadata: None,
        };

        // Test content()
        assert_eq!(chat_completion_chunk.content(), Some("Hello, world!"));

        // Test tool_calls()
        let tool_calls = chat_completion_chunk.tool_calls().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].function.name, "get_current_weather");

        // Test deltas()
        let deltas: Vec<&ChoiceDelta> = chat_completion_chunk.deltas().collect();
        assert_eq!(deltas.len(), 1);
    }
}
