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

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// 在构建参数对象期间可能发生的错误。
#[derive(Error, Debug, PartialEq)]
pub enum ParameterBuilderError {
    #[error("A required property '{0}' is not defined in the properties map")]
    RequiredPropertyNotDefined(String),
}

/// 用于定义工具参数的JSON Schema参数的类型安全表示。(注意这仅仅是在你通过`Parameters::object()`构建才会检查其结构的逻辑合理性, 若你通过其他方式, 比如serde的反序列化来构建则不会保证逻辑合理性)
///
/// 此枚举表示可以定义的不同类型的参数。
/// 每个变体包含一个特定的结构体，用于定义该类型的属性。
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Parameters {
    #[serde(rename = "object")]
    Object(ObjectParameters),
    #[serde(rename = "array")]
    Array(ArrayParameters),
    #[serde(rename = "string")]
    String(StringParameters),
    #[serde(rename = "number")]
    Number(NumberParameters),
    #[serde(rename = "integer")]
    Integer(IntegerParameters),
    #[serde(rename = "boolean")]
    Boolean(BooleanParameters),
}

/// 对象类型的参数。
///
/// 定义具有命名属性的对象。每个属性本身都是一个 `Parameters` 对象。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ObjectParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub properties: HashMap<String, Parameters>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

/// 数组类型的参数。
///
/// 定义一个数组，其中每个项目都符合指定的 `Parameters` 模式。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ArrayParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items: Option<Box<Parameters>>,
}

/// 字符串类型的参数。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct StringParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
}

/// 数字类型（浮点数）的参数。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NumberParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
}

/// 整数类型的参数。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct IntegerParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "enum")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<Value>>,
}

/// 布尔类型的参数。
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct BooleanParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

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
    /// 若你在调用`build()`之前没有通过`property()` 添加属性，则调用`build()`时会返回错误`
    pub fn required(mut self, required: Vec<String>) -> ObjectParametersBuilder {
        *self.params.required.get_or_insert_with(Vec::new) = required;
        self
    }

    /// 将属性标记为必需。
    ///
    /// 若你在调用`build()`之前没有通过`property()` 添加属性，则调用`build()`时会返回错误`
    pub fn require(mut self, name: &str) -> ObjectParametersBuilder {
        self.params
            .required
            .get_or_insert_with(Vec::new)
            .push(name.to_string());
        self
    }
    /// 构建最终的 `Parameters::Object` 实例。
    ///
    /// 此方法执行验证以确保模式是有效的。
    pub fn build(self) -> Result<Parameters, ParameterBuilderError> {
        // 验证所有必需的属性是否存在。
        if let Some(required) = &self.params.required {
            for req_prop in required {
                if !self.params.properties.contains_key(req_prop) {
                    return Err(ParameterBuilderError::RequiredPropertyNotDefined(
                        req_prop.clone(),
                    ));
                }
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

    /// 添加一个字符串枚举值。
    pub fn enum_str(self, value: &str) -> Self {
        self.enum_value(serde_json::json!(value))
    }

    /// 构建最终的 `Parameters::String` 实例。
    pub fn build(self) -> Parameters {
        Parameters::String(self.params)
    }
}

/// 用于构建 `NumberParameters` 实例的构建器。
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

/// 用于构建 `IntegerParameters` 实例的构建器。
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

    /// 为整数添加一个枚举值。
    /// 限制整数必须是所指定值中的一个。
    pub fn enum_int(self, value: i64) -> Self {
        self.enum_value(serde_json::json!(value))
    }

    /// 构建最终的 `Parameters::Integer` 实例。
    pub fn build(self) -> Parameters {
        Parameters::Integer(self.params)
    }
}

/// 用于构建 `BooleanParameters` 实例的构建器。
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
