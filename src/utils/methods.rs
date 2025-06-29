use std::collections::HashMap;

use serde_json::Value;

pub fn merge_extra_metadata(
    left: Option<HashMap<String, Value>>,
    right: Option<HashMap<String, Value>>,
) -> Option<HashMap<String, Value>> {
    match (left, right) {
        (Some(mut left_map), Some(right_map)) => {
            for (key, right_value) in right_map {
                match left_map.get(&key) {
                    Some(left_value) => {
                        left_map.insert(key, merge_json_values(left_value.clone(), right_value));
                    }
                    None => {
                        left_map.insert(key, right_value);
                    }
                }
            }
            Some(left_map)
        }
        (Some(left_map), None) => Some(left_map),
        (None, Some(right_map)) => Some(right_map),
        (None, None) => None,
    }
}

pub fn merge_json_values(left: Value, right: Value) -> Value {
    match (left, right) {
        (Value::Object(mut left_obj), Value::Object(right_obj)) => {
            for (key, right_value) in right_obj {
                match left_obj.get(&key) {
                    Some(left_value) => {
                        left_obj.insert(key, merge_json_values(left_value.clone(), right_value));
                    }
                    None => {
                        left_obj.insert(key, right_value);
                    }
                }
            }
            Value::Object(left_obj)
        }

        (Value::Array(mut left_arr), Value::Array(right_arr)) => {
            left_arr.extend(right_arr);
            Value::Array(left_arr)
        }

        (Value::String(left_str), Value::String(right_str)) => Value::String(left_str + &right_str),

        (Value::Number(left_num), Value::Number(right_num)) => {
            if let (Some(left_f), Some(right_f)) = (left_num.as_f64(), right_num.as_f64()) {
                Value::Number(serde_json::Number::from_f64(left_f + right_f).unwrap_or(left_num))
            } else if let (Some(left_i), Some(right_i)) = (left_num.as_i64(), right_num.as_i64()) {
                Value::Number(serde_json::Number::from(left_i + right_i))
            } else {
                Value::Number(right_num)
            }
        }

        (Value::Bool(left_bool), Value::Bool(right_bool)) => Value::Bool(left_bool || right_bool),

        (_, right) => right,
    }
}
