use serde::ser::SerializeSeq;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub enum Input {
    Text(String),
    List(Vec<String>),
}

#[derive(Debug, Clone, Deserialize)]
pub struct EmbeddingResponse {
    pub model: String,
    #[serde(default = "default_list")]
    pub object: String,
    pub data: Vec<Embedding>,
    pub usage: Usage,
    #[serde(flatten)]
    pub extra_fields: Option<HashMap<String, serde_json::Value>>,
}

impl EmbeddingResponse {
    /// Returns the number of embeddings in the response
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Checks if the response contains any embeddings
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Returns the total number of tokens used in the request
    pub fn total_tokens(&self) -> usize {
        self.usage.total_tokens
    }

    /// Returns the number of prompt tokens used in the request
    pub fn prompt_tokens(&self) -> usize {
        self.usage.prompt_tokens
    }

    /// Returns the embeddings data
    pub fn embeddings(&self) -> &Vec<Embedding> {
        &self.data
    }

    /// Returns a specific embedding by index
    pub fn get_embedding(&self, index: usize) -> Option<&Embedding> {
        self.data.get(index)
    }

    /// Returns all embedding vectors as a vector of float vectors
    pub fn embedding_vectors(&self) -> Vec<&Vec<f32>> {
        self.data.iter().map(|e| &e.embedding).collect()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Embedding {
    pub embedding: Vec<f32>,
    pub index: usize,
    #[serde(default = "default_embedding")]
    pub object: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EncodingFormat {
    #[default]
    Float,
}

fn default_list() -> String {
    "list".to_string()
}

fn default_embedding() -> String {
    "embedding".to_string()
}

impl Embedding {
    /// Returns the dimensionality of the embedding vector
    pub fn dimensions(&self) -> usize {
        self.embedding.len()
    }

    /// Returns a reference to the embedding vector
    pub fn vector(&self) -> &Vec<f32> {
        &self.embedding
    }

    /// Returns the index of this embedding in the response
    pub fn index(&self) -> usize {
        self.index
    }
}

impl Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Input::Text(text) => {
                let mut seq = serializer.serialize_seq(Some(1))?;
                seq.serialize_element(text)?;
                seq.end()
            }
            Input::List(list) => list.serialize(serializer),
        }
    }
}

impl<'a> From<&'a str> for Input {
    fn from(val: &'a str) -> Self {
        Input::Text(val.to_string())
    }
}

impl<'a, T> From<&'a [T]> for Input
where
    T: AsRef<str>,
{
    fn from(slice: &'a [T]) -> Self {
        Input::List(slice.iter().map(|s| s.as_ref().to_string()).collect())
    }
}

impl<T> From<Vec<T>> for Input
where
    T: AsRef<str>,
{
    fn from(vec: Vec<T>) -> Self {
        Input::List(vec.into_iter().map(|s| s.as_ref().to_string()).collect())
    }
}

impl<const N: usize> From<[&str; N]> for Input {
    fn from(val: [&str; N]) -> Self {
        Input::List(val.iter().map(|s| s.to_string()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_into_input() {
        let _input: Input = Input::from("test");
        let _input: Input = Input::from(vec!["test"]);
        let _input: Input = Input::from(["t1", "t2"]);
        let _input: Input = Input::from(vec!["t1".to_string(), "t2".to_string()]);
    }
}
