use super::types::*;
use crate::content;

impl ChatCompletionMessage {
    pub fn is_tool_calls(&self) -> bool {
        self.tool_calls
            .as_ref()
            .map_or(false, |calls| !calls.is_empty())
    }
    pub fn is_reasoning(&self) -> bool {
        self.reasoning
            .as_ref()
            .map_or(false, |reas| !reas.is_empty())
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
            .map_or(false, |calls| !calls.is_empty())
    }

    pub fn is_reasoning(&self) -> bool {
        self.reasoning
            .as_ref()
            .map_or(false, |reas| !reas.is_empty())
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
            parameters: parameters,
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

impl From<ChatCompletionMessageToolCall> for ChatCompletionMessageToolCallParam {
    fn from(value: ChatCompletionMessageToolCall) -> Self {
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
            content: value.content.and_then(|content| Some(content!(content))),
            refusal: value.refusal,
            tool_calls: value.tool_calls.and_then(|tool_calls| {
                Some(
                    tool_calls
                        .into_iter()
                        .map(|tool_call| tool_call.into())
                        .collect(),
                )
            }),
        })
    }
}

impl From<ChoiceDelta> for ChatCompletionMessageParam {
    fn from(value: ChoiceDelta) -> Self {
        Self::Assistant(ChatCompletionAssistantMessageParam {
            name: None,
            content: value.content.and_then(|content| Some(content!(content))),
            refusal: value.refusal,
            tool_calls: value.tool_calls.and_then(|tool_calls| {
                Some(
                    tool_calls
                        .into_iter()
                        .map(|tool_call| tool_call.into())
                        .collect(),
                )
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
