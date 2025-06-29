use super::types::*;
use crate::common::types::{extract_optional, try_deserialize_or_skip};
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt;

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
    use serde_json;
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
}
