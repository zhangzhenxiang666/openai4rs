//! 符合JSON Schema标准的工具参数的类型安全定义。
//!
//! 此模块提供了一个健壮的、类型安全的构建器，用于定义函数工具可以接受的参数。
//! 它确保生成的模式根据OpenAI API支持的JSON Schema子集是有效的。
//!
//! ## 示例
//!
//! ```rust
//! use openai4rs::chat::tool_parameters::Parameters;
//!
//! let params = Parameters::object()
//!     .description("天气函数的参数")
//!     .property(
//!         "location",
//!         Parameters::string().description("城市和州，例如：旧金山，加利福尼亚州").build()
//!     )
//!     .property(
//!         "unit",
//!         Parameters::string()
//!             .description("温度单位")
//!             .enum_str("摄氏度")
//!             .enum_str("华氏度")
//!             .build()
//!     )
//!     .require("location")
//!     .build()
//!     .unwrap();
//! ```
//!
//! 此模块定义了参数类型的层次结构：
//! - `Parameters::Object(ObjectParameters)`: 用于定义具有命名属性的对象。
//! - `Parameters::Array(ArrayParameters)`: 用于定义具有项目类型的数组。
//! - `Parameters::String(StringParameters)`: 用于定义字符串参数。
//! - `Parameters::Number(NumberParameters)`: 用于定义数字参数。
//! - `Parameters::Integer(IntegerParameters)`: 用于定义整数参数。
//! - `Parameters::Boolean(BooleanParameters)`: 用于定义布尔参数。

use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// 在构建参数对象期间可能发生的错误。
#[derive(Error, Debug, PartialEq)]
pub enum ParameterBuilderError {
    #[error("A required property '{0}' is not defined in the properties map")]
    RequiredPropertyNotDefined(String),
}

/// 在将 `serde_json::Value` 转换为 `Parameters` 时可能发生的错误。
#[derive(Error, Debug, PartialEq)]
pub enum ConversionError {
    #[error("The provided JSON Value must be an object, but it is: {0}")]
    ValueNotAnObject(String),
    #[error("The 'type' field is missing from the JSON object")]
    MissingTypeField,
    #[error("The 'type' field must be a string, but it is: {0}")]
    TypeFieldNotAString(String),
    #[error("The value for field '{0}' is invalid: {1}")]
    InvalidFieldValue(String, String),
    #[error("Failed to build parameter object: {0}")]
    BuilderError(#[from] ParameterBuilderError),
}

// --- Core Data Structures ---

/// 用于定义工具参数的JSON Schema参数的类型安全表示。
///
/// 此枚举表示可以定义的不同类型的参数。
/// 每个变体包含一个特定的结构体，用于定义该类型的属性。
#[derive(Debug, Clone, PartialEq)]
pub enum Parameters {
    Object(ObjectParameters),
    Array(ArrayParameters),
    String(StringParameters),
    Number(NumberParameters),
    Integer(IntegerParameters),
    Boolean(BooleanParameters),
}

/// 对象类型的参数。
///
/// 定义具有命名属性的对象。每个属性本身都是一个 `Parameters` 对象。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ObjectParameters {
    pub description: Option<String>,
    pub properties: HashMap<String, Parameters>,
    pub required: Vec<String>,
}

/// 数组类型的参数。
///
/// 定义一个数组，其中每个项目都符合指定的 `Parameters` 模式。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArrayParameters {
    pub description: Option<String>,
    pub items: Option<Box<Parameters>>,
}

/// 字符串类型的参数。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StringParameters {
    pub description: Option<String>,
    pub enum_values: Option<Vec<Value>>,
}

/// 数字类型（浮点数）的参数。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NumberParameters {
    pub description: Option<String>,
    pub enum_values: Option<Vec<Value>>,
}

/// 整数类型的参数。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IntegerParameters {
    pub description: Option<String>,
    pub enum_values: Option<Vec<Value>>,
}

/// 布尔类型的参数。
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BooleanParameters {
    pub description: Option<String>,
}

// --- Builder Implementations ---

/// 用于安全且方便地构建 `ObjectParameters` 实例的构建器。
#[derive(Debug)]
pub struct ObjectParametersBuilder {
    params: ObjectParameters,
}

impl ObjectParametersBuilder {
    fn new() -> ObjectParametersBuilder {
        ObjectParametersBuilder {
            params: ObjectParameters::default(),
        }
    }

    /// 设置对象的描述。
    pub fn description(mut self, description: &str) -> ObjectParametersBuilder {
        self.params.description = Some(description.to_string());
        self
    }

    /// 向对象添加一个属性。
    ///
    /// `name` 是属性名称，`schema` 定义该属性的参数。
    pub fn property(mut self, name: &str, schema: Parameters) -> ObjectParametersBuilder {
        self.params.properties.insert(name.to_string(), schema);
        self
    }

    /// 将属性标记为必需。
    ///
    /// 该属性必须已通过 `property()` 先前添加。
    pub fn required(mut self, required: Vec<String>) -> ObjectParametersBuilder {
        self.params.required = required;
        self
    }

    /// 将属性标记为必需。
    ///
    /// 该属性必须已通过 `property()` 先前添加。
    pub fn require(mut self, name: &str) -> ObjectParametersBuilder {
        self.params.required.push(name.to_string());
        self
    }
    /// 构建最终的 `Parameters::Object` 实例。
    ///
    /// 此方法执行验证以确保模式是有效的。
    pub fn build(self) -> Result<Parameters, ParameterBuilderError> {
        // Validate that all required properties exist
        for req_prop in &self.params.required {
            if !self.params.properties.contains_key(req_prop) {
                return Err(ParameterBuilderError::RequiredPropertyNotDefined(
                    req_prop.clone(),
                ));
            }
        }

        Ok(Parameters::Object(self.params))
    }
}

/// 用于构建 `ArrayParameters` 实例的构建器。
#[derive(Debug)]
pub struct ArrayParametersBuilder {
    params: ArrayParameters,
}

impl ArrayParametersBuilder {
    fn new() -> ArrayParametersBuilder {
        ArrayParametersBuilder {
            params: ArrayParameters::default(),
        }
    }

    /// 设置数组的描述。
    pub fn description(mut self, description: &str) -> ArrayParametersBuilder {
        self.params.description = Some(description.to_string());
        self
    }

    /// 设置数组中项目的模式。
    pub fn items(mut self, items_schema: Parameters) -> ArrayParametersBuilder {
        self.params.items = Some(Box::new(items_schema));
        self
    }

    /// 构建最终的 `Parameters::Array` 实例。
    pub fn build(self) -> Parameters {
        Parameters::Array(self.params)
    }
}

/// 用于构建 `StringParameters` 实例的构建器。
#[derive(Debug)]
pub struct StringParametersBuilder {
    params: StringParameters,
}

impl StringParametersBuilder {
    fn new() -> StringParametersBuilder {
        StringParametersBuilder {
            params: StringParameters::default(),
        }
    }

    /// 设置字符串的描述。
    pub fn description(mut self, description: &str) -> StringParametersBuilder {
        self.params.description = Some(description.to_string());
        self
    }

    /// 为字符串添加一个枚举值。
    ///
    /// 限制字符串必须是所指定值中的一个。
    pub fn enum_value(mut self, value: Value) -> StringParametersBuilder {
        self.params
            .enum_values
            .get_or_insert_with(Vec::new)
            .push(value);
        self
    }

    /// Adds a string enum value.
    ///
    /// A convenience method for adding string enum values.
    pub fn enum_str(self, value: &str) -> Self {
        self.enum_value(serde_json::json!(value))
    }

    /// Builds the final `Parameters::String` instance.
    pub fn build(self) -> Parameters {
        Parameters::String(self.params)
    }
}

/// A builder for constructing `NumberParameters` instances.
#[derive(Debug)]
pub struct NumberParametersBuilder {
    params: NumberParameters,
}

impl NumberParametersBuilder {
    fn new() -> NumberParametersBuilder {
        NumberParametersBuilder {
            params: NumberParameters::default(),
        }
    }

    /// 设置数字的描述。
    pub fn description(mut self, description: &str) -> NumberParametersBuilder {
        self.params.description = Some(description.to_string());
        self
    }

    /// 为数字添加一个枚举值。
    ///
    /// 限制数字必须是所指定值中的一个。
    pub fn enum_value(mut self, value: Value) -> NumberParametersBuilder {
        self.params
            .enum_values
            .get_or_insert_with(Vec::new)
            .push(value);
        self
    }

    /// 构建最终的 `Parameters::Number` 实例。
    pub fn build(self) -> Parameters {
        Parameters::Number(self.params)
    }
}

/// A builder for constructing `IntegerParameters` instances.
#[derive(Debug)]
pub struct IntegerParametersBuilder {
    params: IntegerParameters,
}

impl IntegerParametersBuilder {
    fn new() -> IntegerParametersBuilder {
        IntegerParametersBuilder {
            params: IntegerParameters::default(),
        }
    }

    /// 设置整数的描述。
    pub fn description(mut self, description: &str) -> IntegerParametersBuilder {
        self.params.description = Some(description.to_string());
        self
    }

    /// 为整数添加一个枚举值。
    ///
    /// 限制整数必须是所指定值中的一个。
    pub fn enum_value(mut self, value: Value) -> IntegerParametersBuilder {
        self.params
            .enum_values
            .get_or_insert_with(Vec::new)
            .push(value);
        self
    }

    /// 构建最终的 `Parameters::Integer` 实例。
    pub fn build(self) -> Parameters {
        Parameters::Integer(self.params)
    }
}

/// A builder for constructing `BooleanParameters` instances.
#[derive(Debug)]
pub struct BooleanParametersBuilder {
    params: BooleanParameters,
}

impl BooleanParametersBuilder {
    fn new() -> BooleanParametersBuilder {
        BooleanParametersBuilder {
            params: BooleanParameters::default(),
        }
    }

    /// 设置布尔值的描述。
    pub fn description(mut self, description: &str) -> BooleanParametersBuilder {
        self.params.description = Some(description.to_string());
        self
    }

    /// 构建最终的 `Parameters::Boolean` 实例。
    pub fn build(self) -> Parameters {
        Parameters::Boolean(self.params)
    }
}

// --- Convenience Constructors ---

impl Parameters {
    /// 创建一个新的对象参数构建器。
    pub fn object() -> ObjectParametersBuilder {
        ObjectParametersBuilder::new()
    }

    /// 创建一个新的数组参数构建器。
    pub fn array() -> ArrayParametersBuilder {
        ArrayParametersBuilder::new()
    }

    /// 创建一个新的字符串参数构建器。
    pub fn string() -> StringParametersBuilder {
        StringParametersBuilder::new()
    }

    /// 创建一个新的数字参数构建器。
    pub fn number() -> NumberParametersBuilder {
        NumberParametersBuilder::new()
    }

    /// 创建一个新的整数参数构建器。
    pub fn integer() -> IntegerParametersBuilder {
        IntegerParametersBuilder::new()
    }

    /// 创建一个新的布尔参数构建器。
    pub fn boolean() -> BooleanParametersBuilder {
        BooleanParametersBuilder::new()
    }
}

// --- Serialization ---

impl Serialize for Parameters {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        match self {
            Self::Object(params) => {
                map.serialize_entry("type", "object")?;
                if let Some(ref desc) = params.description {
                    map.serialize_entry("description", desc)?;
                }
                if !params.properties.is_empty() {
                    map.serialize_entry("properties", &params.properties)?;
                }
                if !params.required.is_empty() {
                    map.serialize_entry("required", &params.required)?;
                }
            }
            Self::Array(params) => {
                map.serialize_entry("type", "array")?;
                if let Some(ref desc) = params.description {
                    map.serialize_entry("description", desc)?;
                }
                if let Some(ref items) = params.items {
                    map.serialize_entry("items", items.as_ref())?;
                }
            }
            Self::String(params) => {
                map.serialize_entry("type", "string")?;
                if let Some(ref desc) = params.description {
                    map.serialize_entry("description", desc)?;
                }
                if let Some(ref enum_vals) = params.enum_values {
                    map.serialize_entry("enum", enum_vals)?;
                }
            }
            Self::Number(params) => {
                map.serialize_entry("type", "number")?;
                if let Some(ref desc) = params.description {
                    map.serialize_entry("description", desc)?;
                }
                if let Some(ref enum_vals) = params.enum_values {
                    map.serialize_entry("enum", enum_vals)?;
                }
            }
            Self::Integer(params) => {
                map.serialize_entry("type", "integer")?;
                if let Some(ref desc) = params.description {
                    map.serialize_entry("description", desc)?;
                }
                if let Some(ref enum_vals) = params.enum_values {
                    map.serialize_entry("enum", enum_vals)?;
                }
            }
            Self::Boolean(params) => {
                map.serialize_entry("type", "boolean")?;
                if let Some(ref desc) = params.description {
                    map.serialize_entry("description", desc)?;
                }
            }
        }
        map.end()
    }
}

// --- Conversion from serde_json::Value ---

impl TryFrom<Value> for Parameters {
    type Error = ConversionError;

    fn try_from(value: Value) -> Result<Self, Self::Error> {
        let mut obj = match value {
            Value::Object(map) => map,
            _ => return Err(ConversionError::ValueNotAnObject(format!("{value:?}"))),
        };

        let type_str = obj
            .get("type")
            .and_then(Value::as_str)
            .ok_or(ConversionError::MissingTypeField)?;

        match type_str {
            "object" => {
                let mut builder = ObjectParameters::default();

                if let Some(desc) = obj.get("description").and_then(Value::as_str) {
                    builder.description = Some(desc.to_string());
                }

                if let Some(props_val) = obj.remove("properties") {
                    let props_map = match props_val {
                        Value::Object(map) => map,
                        _ => {
                            return Err(ConversionError::InvalidFieldValue(
                                "properties".to_string(),
                                "must be an object".to_string(),
                            ));
                        }
                    };
                    for (k, v) in props_map {
                        builder.properties.insert(k, Parameters::try_from(v)?);
                    }
                }

                if let Some(req_val) = obj.remove("required") {
                    let req_arr = match req_val {
                        Value::Array(arr) => arr,
                        _ => {
                            return Err(ConversionError::InvalidFieldValue(
                                "required".to_string(),
                                "must be an array".to_string(),
                            ));
                        }
                    };
                    builder.required = req_arr
                        .into_iter()
                        .map(|v| v.as_str().map(ToString::to_string))
                        .collect::<Option<Vec<String>>>()
                        .ok_or_else(|| {
                            ConversionError::InvalidFieldValue(
                                "required".to_string(),
                                "must be an array of strings".to_string(),
                            )
                        })?;
                }

                // Validate that all required properties exist
                for req_prop in &builder.required {
                    if !builder.properties.contains_key(req_prop) {
                        return Err(ConversionError::BuilderError(
                            ParameterBuilderError::RequiredPropertyNotDefined(req_prop.clone()),
                        ));
                    }
                }

                Ok(Parameters::Object(builder))
            }
            "array" => {
                let mut builder = ArrayParameters::default();

                if let Some(desc) = obj.get("description").and_then(Value::as_str) {
                    builder.description = Some(desc.to_string());
                }

                if let Some(items_val) = obj.remove("items") {
                    builder.items = Some(Box::new(Parameters::try_from(items_val)?));
                }

                Ok(Parameters::Array(builder))
            }
            "string" => {
                let mut builder = StringParameters::default();

                if let Some(desc) = obj.get("description").and_then(Value::as_str) {
                    builder.description = Some(desc.to_string());
                }

                if let Some(enum_val) = obj.remove("enum") {
                    let enum_arr = match enum_val {
                        Value::Array(arr) => arr,
                        _ => {
                            return Err(ConversionError::InvalidFieldValue(
                                "enum".to_string(),
                                "must be an array".to_string(),
                            ));
                        }
                    };
                    builder.enum_values = Some(enum_arr);
                }

                Ok(Parameters::String(builder))
            }
            "number" => {
                let mut builder = NumberParameters::default();

                if let Some(desc) = obj.get("description").and_then(Value::as_str) {
                    builder.description = Some(desc.to_string());
                }

                if let Some(enum_val) = obj.remove("enum") {
                    let enum_arr = match enum_val {
                        Value::Array(arr) => arr,
                        _ => {
                            return Err(ConversionError::InvalidFieldValue(
                                "enum".to_string(),
                                "must be an array".to_string(),
                            ));
                        }
                    };
                    builder.enum_values = Some(enum_arr);
                }

                Ok(Parameters::Number(builder))
            }
            "integer" => {
                let mut builder = IntegerParameters::default();

                if let Some(desc) = obj.get("description").and_then(Value::as_str) {
                    builder.description = Some(desc.to_string());
                }

                if let Some(enum_val) = obj.remove("enum") {
                    let enum_arr = match enum_val {
                        Value::Array(arr) => arr,
                        _ => {
                            return Err(ConversionError::InvalidFieldValue(
                                "enum".to_string(),
                                "must be an array".to_string(),
                            ));
                        }
                    };
                    builder.enum_values = Some(enum_arr);
                }

                Ok(Parameters::Integer(builder))
            }
            "boolean" => {
                let mut builder = BooleanParameters::default();

                if let Some(desc) = obj.get("description").and_then(Value::as_str) {
                    builder.description = Some(desc.to_string());
                }

                Ok(Parameters::Boolean(builder))
            }
            _ => Err(ConversionError::TypeFieldNotAString(type_str.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_build_simple_object() {
        let params = Parameters::object()
            .description("A user object")
            .property(
                "name",
                Parameters::string().description("User's name").build(),
            )
            .property(
                "age",
                Parameters::integer().description("User's age").build(),
            )
            .require("name")
            .build()
            .unwrap();

        let json = serde_json::to_value(&params).unwrap();
        let expected = json!({
            "type": "object",
            "description": "A user object",
            "properties": {
                "name": { "type": "string", "description": "User's name" },
                "age": { "type": "integer", "description": "User's age" }
            },
            "required": ["name"]
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_try_from_value_simple_object() {
        let json_value = json!({
            "type": "object",
            "description": "A user object",
            "properties": {
                "name": { "type": "string", "description": "User's name" },
                "age": { "type": "integer", "description": "User's age" }
            },
            "required": ["name"]
        });

        let params = Parameters::try_from(json_value.clone()).unwrap();
        let serialized_params = serde_json::to_value(&params).unwrap();

        assert_eq!(serialized_params, json_value);
    }

    #[test]
    fn test_try_from_value_with_errors() {
        let invalid_json = json!({"type": "object", "required": ["name"]});
        let result = Parameters::try_from(invalid_json);
        assert!(matches!(
            result,
            Err(ConversionError::BuilderError(
                ParameterBuilderError::RequiredPropertyNotDefined(_)
            ))
        ));
    }

    #[test]
    fn test_string_enum() {
        let params = Parameters::string()
            .description("Temperature unit")
            .enum_str("Celsius")
            .enum_str("Fahrenheit")
            .build();

        let json = serde_json::to_value(&params).unwrap();
        let expected = json!({
            "type": "string",
            "description": "Temperature unit",
            "enum": ["Celsius", "Fahrenheit"]
        });
        assert_eq!(json, expected);
    }

    #[test]
    fn test_array_with_items() {
        let params = Parameters::array()
            .description("A list of numbers")
            .items(Parameters::number().description("A number").build())
            .build();

        let json = serde_json::to_value(&params).unwrap();
        let expected = json!({
            "type": "array",
            "description": "A list of numbers",
            "items": {
                "type": "number",
                "description": "A number"
            }
        });
        assert_eq!(json, expected);
    }
}
