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
            ChatCompletionMessageToolCallParam::Function(inner) => {
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
            ChatCompletionMessageParam::System(inner) => {
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
            ChatCompletionMessageParam::User(inner) => {
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
            ChatCompletionMessageParam::Assistant(inner) => {
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
            ChatCompletionMessageParam::Tool(inner) => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assistant_serizalize() {
        let assistant =
            ChatCompletionMessageParam::Assistant(ChatCompletionAssistantMessageParam {
                content: Some(Content::Text("content".into())),
                name: Some("name".into()),
                refusal: Some("refusal".into()),
                tool_calls: Some(vec![ChatCompletionMessageToolCallParam::Function(
                    Function {
                        id: "id".into(),
                        name: "name".into(),
                        arguments: "{'path': '/.cargo'}".into(),
                    },
                )]),
            });
        let json = serde_json::to_string(&assistant).unwrap();
        println!("{}", json);
    }
}
