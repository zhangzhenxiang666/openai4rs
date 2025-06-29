use super::types::*;
use crate::content;
use crate::utils::methods::merge_extra_metadata;

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
