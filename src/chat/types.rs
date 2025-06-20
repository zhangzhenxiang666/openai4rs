use crate::common::types::CompletionGeneric;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Clone, Deserialize)]
pub struct ChoiceDelta {
    pub content: Option<String>,
    pub refusal: Option<String>,
    pub role: Option<String>,
    pub tool_calls: Option<Vec<ChatCompletionToolCall>>,
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolChoice {
    Auto,
    None,
    Required,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ChatCompletionMessage {
    pub content: Option<String>,
    pub refusal: Option<String>,
    pub role: String,
    pub annotations: Option<Vec<Annotation>>,
    pub tool_calls: Option<Vec<ChatCompletionToolCall>>,
    pub reasoning: Option<String>,
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
