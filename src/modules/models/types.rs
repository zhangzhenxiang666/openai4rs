use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct Model {
    pub created: i64,
    pub id: String,
    pub object: Option<String>,
    pub owned_by: Option<String>,
    pub extra_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug)]
pub struct ModelsData {
    pub data: Vec<Model>,
    pub object: Option<String>,
    pub extra_fields: Option<HashMap<String, serde_json::Value>>,
}

impl<'de> serde::Deserialize<'de> for Model {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ModelVisitor;

        impl<'de> serde::de::Visitor<'de> for ModelVisitor {
            type Value = Model;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Model")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut created = None;
                let mut id = None;
                let mut object = None;
                let mut owned_by = None;
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
                        "object" => {
                            if object.is_some() {
                                return Err(serde::de::Error::duplicate_field("object"));
                            }
                            object = Some(map.next_value()?);
                        }
                        "owned_by" => {
                            if owned_by.is_some() {
                                return Err(serde::de::Error::duplicate_field("owned_by"));
                            }
                            owned_by = Some(map.next_value()?);
                        }
                        other => {
                            let value: serde_json::Value = map.next_value()?;
                            extra_fields.insert(other.to_string(), value);
                        }
                    }
                }

                let created = created.ok_or_else(|| serde::de::Error::missing_field("created"))?;
                let id = id.ok_or_else(|| serde::de::Error::missing_field("id"))?;
                let extra_fields = if extra_fields.is_empty() {
                    None
                } else {
                    Some(extra_fields)
                };

                Ok(Model {
                    created,
                    id,
                    object,
                    owned_by,
                    extra_fields,
                })
            }
        }

        deserializer.deserialize_map(ModelVisitor)
    }
}

impl<'de> serde::Deserialize<'de> for ModelsData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ModelsDataVisitor;
        impl<'de> serde::de::Visitor<'de> for ModelsDataVisitor {
            type Value = ModelsData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct ModelsData")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: serde::de::MapAccess<'de>,
            {
                let mut data = None;
                let mut object = None;
                let mut extra_fields = HashMap::new();

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "data" => {
                            if data.is_some() {
                                return Err(serde::de::Error::duplicate_field("data"));
                            }
                            data = Some(map.next_value()?);
                        }
                        "object" => {
                            if object.is_some() {
                                return Err(serde::de::Error::duplicate_field("object"));
                            }
                            object = Some(map.next_value()?);
                        }
                        other => {
                            let value: serde_json::Value = map.next_value()?;
                            extra_fields.insert(other.to_string(), value);
                        }
                    }
                }

                let data = data.ok_or_else(|| serde::de::Error::missing_field("data"))?;
                let extra_fields = if extra_fields.is_empty() {
                    None
                } else {
                    Some(extra_fields)
                };

                Ok(ModelsData {
                    data,
                    object,
                    extra_fields,
                })
            }
        }
        deserializer.deserialize_map(ModelsDataVisitor)
    }
}
