use serde_json::Value;
use std::collections::HashMap;

/// Merges `right` fields into `left` fields in-place.
/// If `left` is `None` and `right` is `Some`, `left` will be replaced by `right`.
/// This avoids unnecessary cloning of the left map when it already exists.
pub fn merge_extra_fields_in_place(
    left: &mut Option<HashMap<String, Value>>,
    right: Option<HashMap<String, Value>>,
) {
    match (left.take(), right) {
        // Both maps exist, merge `right` into `left` and put the result back in `left`.
        (Some(mut left_map), Some(right_map)) => {
            for (key, right_value) in right_map {
                if left_map.contains_key(&key) {
                    let left_value = left_map.remove(&key).unwrap();
                    left_map.insert(key, merge_json_values(left_value, right_value));
                } else {
                    left_map.insert(key, right_value);
                }
            }
            *left = Some(left_map);
        }
        // Only left map exists, put it back as is.
        (Some(left_map), None) => {
            *left = Some(left_map);
        }
        // Only right map exists, or both are None, put right (or None) in left.
        (None, right_map) => {
            *left = right_map;
        }
    }
}

pub fn merge_json_values(left: Value, right: Value) -> Value {
    match (left, right) {
        (Value::Object(mut left_obj), Value::Object(right_obj)) => {
            for (key, right_value) in right_obj {
                if left_obj.contains_key(&key) {
                    let left_value = left_obj.remove(&key).unwrap();
                    left_obj.insert(key, merge_json_values(left_value, right_value));
                } else {
                    left_obj.insert(key, right_value);
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
