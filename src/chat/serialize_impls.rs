use super::types::*;
use serde::Serialize;
use serde_json::Value;

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
                        serde_json::to_value(&content)
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

#[cfg(test)]
mod tests {
    use crate::{chat_request, content, system, user};

    use super::*;

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
            r#"{"model":"meta-llama/llama-3.3-8b-instruct:free","messages":[{"content":"system message","role":"system"},{"content":"user message","role":"user"}],"max_tokens":1024,"n":1,"temperature":0.1,"top_logprobs":1,"tools":[{"function":{"description":"function description","name":"function_name","parameters":{"properties":{"name":{"description":"name of the person","type":"string"}},"required":["name"],"type":"object"},"strict":null},"type":"function"}],"tool_choice":"auto"}"#
        );
    }
}
