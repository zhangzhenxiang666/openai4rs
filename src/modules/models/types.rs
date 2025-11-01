use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Model {
    pub created: i64,
    pub id: String,
    pub object: Option<String>,
    pub owned_by: Option<String>,
    #[serde(flatten)]
    pub extra_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct ModelsData {
    pub data: Vec<Model>,
    pub object: Option<String>,
    #[serde(flatten)]
    pub extra_fields: Option<HashMap<String, serde_json::Value>>,
}
