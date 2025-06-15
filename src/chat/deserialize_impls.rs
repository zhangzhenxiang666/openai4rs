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
                            id = Some(map.next_value()?);
                        }
                        "name" => {
                            if name.is_some() {
                                return Err(de::Error::duplicate_field("name"));
                            }
                            name = Some(map.next_value()?);
                        }
                        "arguments" => {
                            if arguments.is_some() {
                                return Err(de::Error::duplicate_field("arguments"));
                            }
                            arguments = Some(map.next_value()?);
                        }
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let id = id.unwrap_or_default();
                let name = name.ok_or_else(|| de::Error::missing_field("name"))?;
                let arguments = arguments.ok_or_else(|| de::Error::missing_field("arguments"))?;

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

impl<'de> Deserialize<'de> for ChatCompletionMessageToolCall {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ChatCompletionMessageToolCallVisitor;

        impl<'de> Visitor<'de> for ChatCompletionMessageToolCallVisitor {
            type Value = ChatCompletionMessageToolCall;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a ChatCompletionMessageToolCall object")
            }

            fn visit_map<V>(self, mut map: V) -> Result<ChatCompletionMessageToolCall, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut id: Option<String> = None;
                let mut r#type: Option<String> = None;
                let mut function_data: Option<serde_json::Value> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "id" => {
                            if id.is_some() {
                                return Err(de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        "type" => {
                            if r#type.is_some() {
                                return Err(de::Error::duplicate_field("type"));
                            }
                            r#type = Some(map.next_value()?);
                        }
                        "function" => {
                            if function_data.is_some() {
                                return Err(de::Error::duplicate_field("function"));
                            }
                            function_data = Some(map.next_value()?);
                        }
                        _ => {
                            let _: serde_json::Value = map.next_value()?;
                        }
                    }
                }

                let id = id.ok_or_else(|| de::Error::missing_field("id"))?;
                let r#type = r#type.ok_or_else(|| de::Error::missing_field("type"))?;
                let function_data =
                    function_data.ok_or_else(|| de::Error::missing_field("function"))?;

                let mut function: Function = serde_json::from_value(function_data)
                    .map_err(|e| de::Error::custom(format!("Failed to parse function: {}", e)))?;

                function.id = id;

                Ok(ChatCompletionMessageToolCall { function, r#type })
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
        assert!(!chatcompletion.is_err());
    }

    #[test]
    fn test_deserialize_chatcompletion_stream() {
        let json = fs::read_to_string("./assets/chatcompletionchunk.json").unwrap();
        let chatcompletion_chunk: Result<ChatCompletionChunk, _> =
            serde_json::from_str(json.as_str());
        assert!(!chatcompletion_chunk.is_err())
    }
}
