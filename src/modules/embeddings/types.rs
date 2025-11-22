use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
pub enum Input {
    Text(String),
    List(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct EmbeddingResponse {
    pub model: String,
    pub object: String,
    pub data: Vec<Embedding>,
    pub usage: Usage,
    pub extra_fields: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Clone)]
pub enum EmbeddingData {
    Float(Vec<f32>),
    Base64(String),
}

#[derive(Debug, Clone)]
pub struct Embedding {
    pub embedding: EmbeddingData,
    pub index: usize,
    pub object: String,
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EncodingFormat {
    #[default]
    Float,
    Base64,
}

impl EmbeddingResponse {
    /// 返回响应中的嵌入数量
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// 检查响应是否包含任何嵌入
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// 返回请求中使用的令牌总数
    pub fn total_tokens(&self) -> usize {
        self.usage.total_tokens
    }

    /// 返回请求中使用的提示令牌数量
    pub fn prompt_tokens(&self) -> usize {
        self.usage.prompt_tokens
    }

    /// 返回嵌入数据
    pub fn embeddings(&self) -> &Vec<Embedding> {
        &self.data
    }

    /// 根据索引返回特定的嵌入
    pub fn get_embedding(&self, index: usize) -> Option<&Embedding> {
        self.data.get(index)
    }

    /// 将所有嵌入向量作为浮点向量的向量返回，忽略任何base64编码的嵌入
    pub fn embedding_vectors(&self) -> Vec<Vec<f32>> {
        self.data
            .iter()
            .filter_map(|e| match &e.embedding {
                EmbeddingData::Float(vec) => Some(vec.clone()),
                EmbeddingData::Base64(_) => None,
            })
            .collect()
    }

    /// 将所有嵌入作为浮点向量返回，必要时尝试解码base64
    pub fn embedding_vectors_decoded(&self) -> Vec<Vec<f32>> {
        self.data.iter().filter_map(|e| e.vector()).collect()
    }
}

impl Embedding {
    /// 返回嵌入向量的维度
    pub fn dimensions(&self) -> usize {
        match &self.embedding {
            EmbeddingData::Float(vec) => vec.len(),
            EmbeddingData::Base64(_) => {
                // 对于base64，我们可以解码它以获取实际的浮点数计数
                // 目前，返回0或我们可以实现适当的解码
                0
            }
        }
    }

    /// 将嵌入向量作为浮点向量返回，必要时尝试从base64解码
    pub fn vector(&self) -> Option<Vec<f32>> {
        match &self.embedding {
            EmbeddingData::Float(vec) => Some(vec.clone()),
            EmbeddingData::Base64(base64_str) => {
                // 尝试将base64解码为浮点向量
                decode_base64_embedding(base64_str)
            }
        }
    }

    /// 返回此嵌入在响应中的索引
    pub fn index(&self) -> usize {
        self.index
    }

    /// 返回嵌入数据为base64字符串（如果可用）
    pub fn as_base64(&self) -> Option<&str> {
        match &self.embedding {
            EmbeddingData::Base64(base64_str) => Some(base64_str),
            _ => None,
        }
    }

    /// 返回嵌入数据为浮点向量（如果可用）
    pub fn as_float(&self) -> Option<&Vec<f32>> {
        match &self.embedding {
            EmbeddingData::Float(vec) => Some(vec),
            _ => None,
        }
    }

    /// 将嵌入数据转换为浮点向量（如果可用）
    pub fn to_float(self) -> Option<Vec<f32>> {
        match self.embedding {
            EmbeddingData::Float(vec) => Some(vec),
            EmbeddingData::Base64(base64_str) => decode_base64_embedding(base64_str.as_str()),
        }
    }
}

/// 将base64编码的嵌入数据解码为浮点向量的辅助函数
fn decode_base64_embedding(base64_str: &str) -> Option<Vec<f32>> {
    use base64::Engine;
    use base64::engine::general_purpose;
    match general_purpose::STANDARD.decode(base64_str) {
        Ok(decoded_bytes) => {
            // 将字节转换为f32切片 - 这假设数据序列化为f32字节
            // 这可能需要根据OpenAI实际编码嵌入的方式进行调整
            if decoded_bytes.len() % std::mem::size_of::<f32>() == 0 {
                // 这是一个简化的转换 - 实际上，我们需要正确处理字节顺序
                let float_count = decoded_bytes.len() / std::mem::size_of::<f32>();
                let mut result = Vec::with_capacity(float_count);

                for chunk in decoded_bytes.chunks_exact(std::mem::size_of::<f32>()) {
                    let bytes: [u8; 4] = [chunk[0], chunk[1], chunk[2], chunk[3]];
                    result.push(f32::from_le_bytes(bytes)); // 假设小端字节序
                }
                Some(result)
            } else {
                None
            }
        }
        Err(_) => None,
    }
}

impl Serialize for Input {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Input::Text(text) => serializer.serialize_str(text),
            Input::List(list) => list.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for Embedding {
    fn deserialize<D>(deserializer: D) -> Result<Embedding, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EmbeddingVisitor;

        impl<'de> Visitor<'de> for EmbeddingVisitor {
            type Value = Embedding;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Embedding")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Embedding, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut embedding = None;
                let mut index = None;
                let mut object = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "embedding" => {
                            if embedding.is_some() {
                                return Err(de::Error::duplicate_field("embedding"));
                            }
                            embedding = Some(map.next_value()?);
                        }
                        "index" => {
                            if index.is_some() {
                                return Err(de::Error::duplicate_field("index"));
                            }
                            index = Some(map.next_value()?);
                        }
                        "object" => {
                            if object.is_some() {
                                return Err(de::Error::duplicate_field("object"));
                            }
                            object = Some(map.next_value()?);
                        }
                        _ => {
                            let _ = map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                let embedding = embedding.ok_or_else(|| de::Error::missing_field("embedding"))?;
                let index = index.ok_or_else(|| de::Error::missing_field("index"))?;
                let object = object.unwrap_or_else(|| "embedding".to_string());

                Ok(Embedding {
                    embedding,
                    index,
                    object,
                })
            }
        }

        deserializer.deserialize_struct(
            "Embedding",
            &["embedding", "index", "object"],
            EmbeddingVisitor,
        )
    }
}

impl<'de> serde::Deserialize<'de> for EmbeddingData {
    fn deserialize<D>(deserializer: D) -> Result<EmbeddingData, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct EmbeddingDataVisitor;

        impl<'de> Visitor<'de> for EmbeddingDataVisitor {
            type Value = EmbeddingData;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a float array or a base64 string")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<EmbeddingData, A::Error>
            where
                A: de::SeqAccess<'de>,
            {
                let mut vec = Vec::new();
                while let Some(value) = seq.next_element::<f32>()? {
                    vec.push(value);
                }
                Ok(EmbeddingData::Float(vec))
            }

            fn visit_str<E>(self, value: &str) -> Result<EmbeddingData, E>
            where
                E: de::Error,
            {
                // For now, we'll assume string values are base64 format
                Ok(EmbeddingData::Base64(value.to_string()))
            }
        }

        deserializer.deserialize_any(EmbeddingDataVisitor)
    }
}

impl<'de> Deserialize<'de> for EmbeddingResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EmbeddingResponseVisitor;

        impl<'de> Visitor<'de> for EmbeddingResponseVisitor {
            type Value = EmbeddingResponse;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct EmbeddingResponse")
            }

            fn visit_map<V>(self, mut map: V) -> Result<EmbeddingResponse, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut model = None;
                let mut object = None;
                let mut data = None;
                let mut usage = None;
                let mut extra_fields: Option<HashMap<String, serde_json::Value>> = None;

                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "model" => {
                            if model.is_some() {
                                return Err(de::Error::duplicate_field("model"));
                            }
                            model = Some(map.next_value()?);
                        }
                        "object" => {
                            if object.is_some() {
                                return Err(de::Error::duplicate_field("object"));
                            }
                            object = Some(map.next_value()?);
                        }
                        "data" => {
                            if data.is_some() {
                                return Err(de::Error::duplicate_field("data"));
                            }
                            data = Some(map.next_value()?);
                        }
                        "usage" => {
                            if usage.is_some() {
                                return Err(de::Error::duplicate_field("usage"));
                            }
                            usage = Some(map.next_value()?);
                        }
                        _ => {
                            let value = map.next_value()?;
                            extra_fields
                                .get_or_insert_with(HashMap::new)
                                .insert(key, value);
                        }
                    }
                }

                let model = model.ok_or_else(|| de::Error::missing_field("model"))?;
                let object = object.unwrap_or_else(|| "list".to_string());
                let data = data.ok_or_else(|| de::Error::missing_field("data"))?;
                let usage = usage.ok_or_else(|| de::Error::missing_field("usage"))?;

                Ok(EmbeddingResponse {
                    model,
                    object,
                    data,
                    usage,
                    extra_fields,
                })
            }
        }

        deserializer.deserialize_map(EmbeddingResponseVisitor)
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
    use base64::Engine;
    use serde_json;

    #[test]
    fn test_into_input() {
        let _input: Input = Input::from("test");
        let _input: Input = Input::from(vec!["test"]);
        let _input: Input = Input::from(["t1", "t2"]);
        let _input: Input = Input::from(vec!["t1".to_string(), "t2".to_string()]);
    }

    #[test]
    fn test_encoding_format_serialization() {
        assert_eq!(
            serde_json::to_string(&EncodingFormat::Float).unwrap(),
            "\"float\""
        );
        assert_eq!(
            serde_json::to_string(&EncodingFormat::Base64).unwrap(),
            "\"base64\""
        );
    }

    #[test]
    fn test_embedding_data_deserialize_float() {
        let json = "[0.1, 0.2, 0.3]";
        let result: EmbeddingData = serde_json::from_str(json).unwrap();

        match result {
            EmbeddingData::Float(vec) => {
                assert_eq!(vec, vec![0.1, 0.2, 0.3]);
            }
            EmbeddingData::Base64(_) => panic!("Expected Float variant"),
        }
    }

    #[test]
    fn test_embedding_data_deserialize_base64() {
        let json = "\"SGVsbG8gV29ybGQ=\"";
        let result: EmbeddingData = serde_json::from_str(json).unwrap();

        match result {
            EmbeddingData::Base64(s) => {
                assert_eq!(s, "SGVsbG8gV29ybGQ=");
            }
            EmbeddingData::Float(_) => panic!("Expected Base64 variant"),
        }
    }

    #[test]
    fn test_embedding_deserialize() {
        let json = r#"{
            "embedding": [0.1, 0.2, 0.3],
            "index": 0,
            "object": "embedding"
        }"#;

        let result: Embedding = serde_json::from_str(json).unwrap();
        assert_eq!(result.index, 0);
        assert_eq!(result.object, "embedding");

        match result.embedding {
            EmbeddingData::Float(vec) => {
                assert_eq!(vec, vec![0.1, 0.2, 0.3]);
            }
            EmbeddingData::Base64(_) => panic!("Expected Float variant"),
        }
    }

    #[test]
    fn test_embedding_with_base64_data() {
        let json = r#"{
            "embedding": "SGVsbG8gV29ybGQ=",
            "index": 0,
            "object": "embedding"
        }"#;

        let result: Embedding = serde_json::from_str(json).unwrap();
        assert_eq!(result.index, 0);
        assert_eq!(result.object, "embedding");

        match result.embedding {
            EmbeddingData::Base64(s) => {
                assert_eq!(s, "SGVsbG8gV29ybGQ=");
            }
            EmbeddingData::Float(_) => panic!("Expected Base64 variant"),
        }
    }

    #[test]
    fn test_decode_base64_embedding() {
        // Create a simple test with some float values and encode them to base64
        let original_values = vec![1.0f32, 2.0f32, 3.0f32];
        let bytes: Vec<u8> = original_values
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();
        let base64_str = base64::engine::general_purpose::STANDARD.encode(&bytes);

        let decoded = decode_base64_embedding(&base64_str);
        assert!(decoded.is_some());
        let decoded_values = decoded.unwrap();
        assert_eq!(decoded_values.len(), 3);
        assert!((decoded_values[0] - 1.0).abs() < f32::EPSILON);
        assert!((decoded_values[1] - 2.0).abs() < f32::EPSILON);
        assert!((decoded_values[2] - 3.0).abs() < f32::EPSILON);
    }
}
