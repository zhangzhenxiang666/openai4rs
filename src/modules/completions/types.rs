use crate::common::types::CompletionGeneric;
use serde::Deserialize;
use serde::de::{self, MapAccess, Visitor};
use std::collections::HashMap;
use std::fmt;

pub type Completion = CompletionGeneric<CompletionChoice>;

#[derive(Debug, Clone)]
pub struct CompletionChoice {
    pub index: usize,
    pub text: String,
    pub finish_reason: Option<FinishReason>,
    pub logprobs: Option<Logprobs>,
    pub reasoning: Option<String>,
    pub extra_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Logprobs {
    pub text_offset: Option<Vec<i64>>,
    pub token_logprobs: Option<Vec<f64>>,
    pub tokens: Option<Vec<String>>,
    pub top_logprobs: Option<Vec<HashMap<String, f64>>>,
}

impl CompletionChoice {
    pub fn is_reasoning(&self) -> bool {
        self.reasoning.as_ref().is_some_and(|reas| !reas.is_empty())
    }

    pub fn get_reasoning_str(&self) -> &str {
        match self.reasoning.as_ref() {
            Some(reasoning) => reasoning.as_str(),
            None => "",
        }
    }

    pub fn get_text_str(&self) -> &str {
        self.text.as_str()
    }
}

impl<'de> Deserialize<'de> for CompletionChoice {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CompletionChoiceVisitor;

        impl<'de> Visitor<'de> for CompletionChoiceVisitor {
            type Value = CompletionChoice;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a CompletionChoice object")
            }

            fn visit_map<V>(self, mut map: V) -> Result<CompletionChoice, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut index: Option<usize> = None;
                let mut text: Option<String> = None;
                let mut finish_reason: Option<Option<FinishReason>> = None;
                let mut logprobs: Option<Option<Logprobs>> = None;
                let mut reasoning: Option<Option<String>> = None;
                let mut reasoning_content: Option<Option<String>> = None;
                let mut extra_fields: Option<HashMap<String, serde_json::Value>> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "index" => {
                            if index.is_some() {
                                return Err(de::Error::duplicate_field("index"));
                            }
                            index = Some(map.next_value()?);
                        }
                        "text" => {
                            if text.is_some() {
                                return Err(de::Error::duplicate_field("text"));
                            }
                            text = Some(map.next_value()?);
                        }
                        "finish_reason" => {
                            if finish_reason.is_some() {
                                return Err(de::Error::duplicate_field("finish_reason"));
                            }
                            finish_reason = Some(map.next_value()?);
                        }
                        "logprobs" => {
                            if logprobs.is_some() {
                                return Err(de::Error::duplicate_field("logprobs"));
                            }
                            logprobs = Some(map.next_value()?);
                        }
                        "reasoning" => {
                            if reasoning.is_some() {
                                return Err(de::Error::duplicate_field("reasoning"));
                            }
                            reasoning = Some(map.next_value()?);
                        }
                        "reasoning_content" => {
                            if reasoning_content.is_some() {
                                return Err(de::Error::duplicate_field("reasoning_content"));
                            }
                            reasoning_content = Some(map.next_value()?);
                        }
                        _ => {
                            let value = map.next_value()?;
                            extra_fields
                                .get_or_insert_with(HashMap::new)
                                .insert(key, value);
                        }
                    }
                }

                let final_reasoning = reasoning.flatten().or(reasoning_content.flatten());

                Ok(CompletionChoice {
                    index: index.unwrap_or(0),
                    text: text.unwrap_or_default(),
                    finish_reason: finish_reason.flatten(),
                    logprobs: logprobs.flatten(),
                    reasoning: final_reasoning,
                    extra_fields,
                })
            }
        }

        deserializer.deserialize_map(CompletionChoiceVisitor)
    }
}
