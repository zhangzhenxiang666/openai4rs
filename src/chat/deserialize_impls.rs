use super::types::*;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
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
                            if let Ok(value) = map.next_value::<Option<String>>() {
                                id = value;
                            } else {
                                let _: serde_json::Value = map.next_value()?;
                            }
                        }
                        "name" => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            if let Ok(value) = map.next_value::<Option<String>>() {
                                name = value;
                            } else {
                                let _: serde_json::Value = map.next_value()?;
                            }
                        }
                        "arguments" => {
                            if arguments.is_some() {
                                return Err(de::Error::duplicate_field("arguments"));
                            }
                            if let Ok(value) = map.next_value::<Option<String>>() {
                                arguments = value;
                            } else {
                                let _: serde_json::Value = map.next_value()?;
                            }
                        }
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let id = id.unwrap_or_default();
                let name = name.unwrap_or_default();
                let arguments = arguments.unwrap_or_default();

                Ok(Function {
                    id,
                    name,
                    arguments,
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
        struct ChatCompletionMessageToolCallVisitor;

        impl<'de> Visitor<'de> for ChatCompletionMessageToolCallVisitor {
            type Value = ChatCompletionToolCall;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a ChatCompletionMessageToolCall object")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ChatCompletionToolCall, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id: Option<String> = None;
                let mut index = 0;
                let mut r#type: Option<String> = None;
                let mut function_data: Option<serde_json::Value> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = map.next_value::<Option<String>>()?;
                        }
                        "type" => {
                            if r#type.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }
                            r#type = map.next_value::<Option<String>>()?;
                        }
                        "function" => {
                            if function_data.is_some() {
                                return Err(de::Error::duplicate_field("function"));
                            }
                            function_data = map.next_value::<Option<serde_json::Value>>()?;
                        }
                        "index" => {
                            if index != 0 {
                                return Err(de::Error::duplicate_field("index"));
                            }
                            if let Some(idx) = map.next_value::<Option<i64>>()? {
                                index = idx;
                            }
                        }
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let id = id.unwrap_or_default();
                let r#type = r#type.ok_or_else(|| de::Error::missing_field("type"))?;

                let function_data = function_data.unwrap_or_else(|| {
                    serde_json::json!({
                        "id": "",
                        "name": "",
                        "arguments": ""
                    })
                });

                let mut function: Function = serde_json::from_value(function_data)
                    .map_err(|e| de::Error::custom(format!("Failed to parse function: {}", e)))?;

                function.id = id;

                Ok(ChatCompletionToolCall {
                    function,
                    r#type,
                    index,
                })
            }
        }

        deserializer.deserialize_map(ChatCompletionMessageToolCallVisitor)
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
        assert!(chatcompletion_chunk.is_ok())
    }
}
