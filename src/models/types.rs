use std::collections::HashMap;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Model {
    pub id: String,
    pub created: i64,
    pub object: Option<String>,
    pub owned_by: Option<String>,
    #[serde(flatten)]
    pub extra_metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
pub struct ModelsData {
    pub object: Option<String>,
    pub data: Vec<Model>,
    #[serde(flatten)]
    pub extra_metadata: Option<HashMap<String, serde_json::Value>>,
}
