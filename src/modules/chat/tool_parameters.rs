//! Type-safe definitions for building tool parameters that conform to the JSON Schema standard.
//!
//! This module provides a robust, type-safe builder for defining the parameters a function
//! tool can accept. It ensures that the generated schema is valid according to the subset
//! of JSON Schema supported by OpenAI's API.
//!
//! ## Example
//!
//! ```rust
//! use openai4rs::chat::tool_parameters::Parameters;
//!
//! let params = Parameters::object()
//!     .description("Parameters for the weather function")
//!     .property(
//!         "location",
//!         Parameters::string().description("The city and state, e.g., San Francisco, CA").build()
//!     )
//!     .property(
//!         "unit",
//!         Parameters::string()
//!             .description("The unit of temperature")
//!             .enum_str("Celsius")
//!             .enum_str("Fahrenheit")
//!             .build()
//!     )
//!     .require("location")
//!     .build()
//!     .unwrap();
//! ```
//!
//! This module defines a hierarchy of parameter types:
//! - `Parameters::Object(ObjectParameters)`: For defining objects with named properties.
//! - `Parameters::Array(ArrayParameters)`: For defining arrays with item types.
//! - `Parameters::String(StringParameters)`: For defining string parameters.
//! - `Parameters::Number(NumberParameters)`: For defining number parameters.
//! - `Parameters::Integer(IntegerParameters)`: For defining integer parameters.
//! - `Parameters::Boolean(BooleanParameters)`: For defining boolean parameters.

use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

/// An error that can occur during the construction of parameter objects.
#[derive(Error, Debug, PartialEq)]
pub enum ParameterBuilderError {
    #[error("A required property '{0}' is not defined in the properties map")]
    RequiredPropertyNotDefined(String),
}

/// An error that can occur when converting a `serde_json::Value` to `Parameters`.
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

/// A type-safe representation of JSON Schema parameters for defining tool parameters.
///
/// This enum represents the different types of parameters that can be defined.
/// Each variant contains a specific struct that defines the properties for that type.
#[derive(Debug, Clone, PartialEq)]
pub enum Parameters {
    Object(ObjectParameters),
    Array(ArrayParameters),
    String(StringParameters),
    Number(NumberParameters),
    Integer(IntegerParameters),
    Boolean(BooleanParameters),
}

/// Parameters for an object type.
///
/// Defines an object with named properties. Each property is itself a `Parameters` object.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ObjectParameters {
    pub description: Option<String>,
    pub properties: HashMap<String, Parameters>,
    pub required: Vec<String>,
}

/// Parameters for an array type.
///
/// Defines an array where each item conforms to a specified `Parameters` schema.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ArrayParameters {
    pub description: Option<String>,
    pub items: Option<Box<Parameters>>,
}

/// Parameters for a string type.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StringParameters {
    pub description: Option<String>,
    pub enum_values: Option<Vec<Value>>,
}

/// Parameters for a number type (floating point).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NumberParameters {
    pub description: Option<String>,
    pub enum_values: Option<Vec<Value>>,
}

/// Parameters for an integer type.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct IntegerParameters {
    pub description: Option<String>,
    pub enum_values: Option<Vec<Value>>,
}

/// Parameters for a boolean type.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BooleanParameters {
    pub description: Option<String>,
}

// --- Builder Implementations ---

/// A builder for constructing `ObjectParameters` instances safely and conveniently.
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

    /// Sets the description for the object.
    pub fn description(mut self, description: &str) -> ObjectParametersBuilder {
        self.params.description = Some(description.to_string());
        self
    }

    /// Adds a property to the object.
    ///
    /// The `name` is the property name, and `schema` defines the parameter for that property.
    pub fn property(mut self, name: &str, schema: Parameters) -> ObjectParametersBuilder {
        self.params.properties.insert(name.to_string(), schema);
        self
    }

    /// Marks a property as required.
    ///
    /// The property must have been previously added with `property()`.
    pub fn required(mut self, required: Vec<String>) -> ObjectParametersBuilder {
        self.params.required = required;
        self
    }

    /// Marks a property as require.
    ///
    /// The property must have been previously added with `property()`.
    pub fn require(mut self, name: &str) -> ObjectParametersBuilder {
        self.params.required.push(name.to_string());
        self
    }
    /// Builds the final `Parameters::Object` instance.
    ///
    /// This method performs validation to ensure the schema is valid.
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

/// A builder for constructing `ArrayParameters` instances.
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

    /// Sets the description for the array.
    pub fn description(mut self, description: &str) -> ArrayParametersBuilder {
        self.params.description = Some(description.to_string());
        self
    }

    /// Sets the schema for the items in the array.
    pub fn items(mut self, items_schema: Parameters) -> ArrayParametersBuilder {
        self.params.items = Some(Box::new(items_schema));
        self
    }

    /// Builds the final `Parameters::Array` instance.
    pub fn build(self) -> Parameters {
        Parameters::Array(self.params)
    }
}

/// A builder for constructing `StringParameters` instances.
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

    /// Sets the description for the string.
    pub fn description(mut self, description: &str) -> StringParametersBuilder {
        self.params.description = Some(description.to_string());
        self
    }

    /// Adds an enum value for the string.
    ///
    /// Constrains the string to be one of the specified values.
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

    /// Sets the description for the number.
    pub fn description(mut self, description: &str) -> NumberParametersBuilder {
        self.params.description = Some(description.to_string());
        self
    }

    /// Adds an enum value for the number.
    ///
    /// Constrains the number to be one of the specified values.
    pub fn enum_value(mut self, value: Value) -> NumberParametersBuilder {
        self.params
            .enum_values
            .get_or_insert_with(Vec::new)
            .push(value);
        self
    }

    /// Builds the final `Parameters::Number` instance.
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

    /// Sets the description for the integer.
    pub fn description(mut self, description: &str) -> IntegerParametersBuilder {
        self.params.description = Some(description.to_string());
        self
    }

    /// Adds an enum value for the integer.
    ///
    /// Constrains the integer to be one of the specified values.
    pub fn enum_value(mut self, value: Value) -> IntegerParametersBuilder {
        self.params
            .enum_values
            .get_or_insert_with(Vec::new)
            .push(value);
        self
    }

    /// Builds the final `Parameters::Integer` instance.
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

    /// Sets the description for the boolean.
    pub fn description(mut self, description: &str) -> BooleanParametersBuilder {
        self.params.description = Some(description.to_string());
        self
    }

    /// Builds the final `Parameters::Boolean` instance.
    pub fn build(self) -> Parameters {
        Parameters::Boolean(self.params)
    }
}

// --- Convenience Constructors ---

impl Parameters {
    /// Creates a new object parameters builder.
    pub fn object() -> ObjectParametersBuilder {
        ObjectParametersBuilder::new()
    }

    /// Creates a new array parameters builder.
    pub fn array() -> ArrayParametersBuilder {
        ArrayParametersBuilder::new()
    }

    /// Creates a new string parameters builder.
    pub fn string() -> StringParametersBuilder {
        StringParametersBuilder::new()
    }

    /// Creates a new number parameters builder.
    pub fn number() -> NumberParametersBuilder {
        NumberParametersBuilder::new()
    }

    /// Creates a new integer parameters builder.
    pub fn integer() -> IntegerParametersBuilder {
        IntegerParametersBuilder::new()
    }

    /// Creates a new boolean parameters builder.
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
            _ => return Err(ConversionError::ValueNotAnObject(format!("{:?}", value))),
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
