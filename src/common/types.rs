use http::{Extensions, HeaderMap};
use serde::{Deserialize, Serialize, de::MapAccess};
use std::collections::HashMap;
use std::fmt;
use std::marker::PhantomData;

#[derive(Debug, Clone)]
pub struct CompletionGeneric<T> {
    pub created: i64,
    pub id: String,
    pub model: String,
    pub object: String,
    pub choices: Vec<T>,
    pub service_tier: Option<ServiceTier>,
    pub system_fingerprint: Option<String>,
    pub usage: Option<CompletionUsage>,
    pub extra_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionUsage {
    pub completion_tokens: i64,
    pub prompt_tokens: i64,
    pub total_tokens: i64,
    pub completion_tokens_details: Option<CompletionTokensDetails>,
    pub prompt_tokens_details: Option<PromptTokensDetails>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CompletionTokensDetails {
    pub accepted_prediction_tokens: Option<i64>,
    pub audio_tokens: Option<i64>,
    pub reasoning_tokens: Option<i64>,
    pub rejected_prediction_tokens: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PromptTokensDetails {
    pub audio_tokens: Option<i64>,
    pub cached_tokens: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ServiceTier {
    Auto,
    Default,
}

pub(crate) type JsonBody = serde_json::Map<String, serde_json::Value>;

#[derive(Debug, Clone)]
pub(crate) struct Timeout(pub std::time::Duration);

#[derive(Debug, Clone)]
pub(crate) struct RetryCount(pub usize);

pub(crate) struct InParam {
    pub body: Option<JsonBody>,
    pub headers: HeaderMap,
    pub extensions: Extensions,
}

impl InParam {
    pub(crate) fn new() -> Self {
        Self {
            body: None,
            headers: HeaderMap::new(),
            extensions: Extensions::new(),
        }
    }
}

pub(crate) fn try_deserialize_or_skip<'de, T, V>(map: &mut V) -> Result<Option<T>, V::Error>
where
    T: serde::de::DeserializeOwned,
    V: MapAccess<'de>,
{
    match map.next_value::<Option<T>>() {
        Ok(value) => Ok(value),
        Err(e) => match map.next_value::<serde_json::Value>() {
            Ok(_) => Ok(None),
            Err(_) => Err(e),
        },
    }
}

impl<'de, T> serde::Deserialize<'de> for CompletionGeneric<T>
where
    T: serde::Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct CompletionGenericVisitor<TI> {
            _marker: PhantomData<TI>,
        }

        impl<'de, TI> serde::de::Visitor<'de> for CompletionGenericVisitor<TI>
        where
            TI: serde::Deserialize<'de>,
        {
            type Value = CompletionGeneric<TI>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct CompletionGeneric")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut created = None;
                let mut id = None;
                let mut model = None;
                let mut object = None;
                let mut choices = None;
                let mut service_tier = None;
                let mut system_fingerprint = None;
                let mut usage = None;
                let mut extra_fields = HashMap::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "created" => {
                            if created.is_some() {
                                return Err(serde::de::Error::duplicate_field("created"));
                            }
                            created = Some(map.next_value()?);
                        }
                        "id" => {
                            if id.is_some() {
                                return Err(serde::de::Error::duplicate_field("id"));
                            }
                            id = Some(map.next_value()?);
                        }
                        "model" => {
                            if model.is_some() {
                                return Err(serde::de::Error::duplicate_field("model"));
                            }
                            model = Some(map.next_value()?);
                        }
                        "object" => {
                            if object.is_some() {
                                return Err(serde::de::Error::duplicate_field("object"));
                            }
                            object = Some(map.next_value()?);
                        }
                        "choices" => {
                            if choices.is_some() {
                                return Err(serde::de::Error::duplicate_field("choices"));
                            }
                            choices = Some(map.next_value()?);
                        }
                        "service_tier" => {
                            if service_tier.is_some() {
                                return Err(serde::de::Error::duplicate_field("service_tier"));
                            }
                            service_tier = Some(map.next_value()?);
                        }
                        "system_fingerprint" => {
                            if system_fingerprint.is_some() {
                                return Err(serde::de::Error::duplicate_field(
                                    "system_fingerprint",
                                ));
                            }
                            system_fingerprint = Some(map.next_value()?);
                        }
                        "usage" => {
                            if usage.is_some() {
                                return Err(serde::de::Error::duplicate_field("usage"));
                            }
                            usage = Some(map.next_value()?);
                        }
                        _ => {
                            let value = map.next_value()?;
                            extra_fields.insert(key, value);
                        }
                    }
                }

                let created = created.ok_or_else(|| serde::de::Error::missing_field("created"))?;
                let id = id.unwrap_or_else(|| "0".to_string());
                let model = model.ok_or_else(|| serde::de::Error::missing_field("model"))?;
                let object = object.ok_or_else(|| serde::de::Error::missing_field("object"))?;
                let choices = choices.ok_or_else(|| serde::de::Error::missing_field("choices"))?;

                let extra_fields = if extra_fields.is_empty() {
                    None
                } else {
                    Some(extra_fields)
                };

                Ok(CompletionGeneric {
                    created,
                    id,
                    model,
                    object,
                    choices,
                    service_tier,
                    system_fingerprint,
                    usage,
                    extra_fields,
                })
            }
        }

        deserializer.deserialize_map(CompletionGenericVisitor {
            _marker: PhantomData,
        })
    }
}
