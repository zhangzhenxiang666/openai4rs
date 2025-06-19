#[macro_export]
macro_rules! content {
    ($input:tt) => {{
        let value = serde_json::json!($input);
        match value {
            serde_json::Value::Object(_) => $crate::Content::Object(value),
            serde_json::Value::String(s) => $crate::Content::Text(s),
            serde_json::Value::Array(_) => $crate::Content::Object(value),
            serde_json::Value::Number(n) => $crate::Content::Text(n.to_string()),
            serde_json::Value::Bool(b) => $crate::Content::Text(b.to_string()),
            serde_json::Value::Null => $crate::Content::Text("null".to_string()),
        }
    }};
}

#[macro_export]
macro_rules! system {
    ($content:tt) => {{ $crate::system!($content, None) }};

    ($content:tt, None) => {{
        $crate::ChatCompletionMessageParam::System($crate::ChatCompletionSystemMessageParam {
            content: $crate::content!($content),
            name: None,
        })
    }};

    ($content:tt, $name:expr) => {{
        fn check_name<T: AsRef<str>>(name: T) -> String {
            name.as_ref().to_string()
        }

        $crate::ChatCompletionMessageParam::System($crate::ChatCompletionSystemMessageParam {
            content: $crate::content!($content),
            name: Some(check_name($name)),
        })
    }};
}

#[macro_export]
macro_rules! user {
    ($content:tt) => {{ $crate::user!($content, None) }};

    ($content:tt, None) => {{
        $crate::ChatCompletionMessageParam::User($crate::ChatCompletionUserMessageParam {
            content: $crate::content!($content),
            name: None,
        })
    }};

    ($content:tt, $name:expr) => {{
        fn check_name<T: AsRef<str>>(name: T) -> String {
            name.as_ref().to_string()
        }

        $crate::ChatCompletionMessageParam::User($crate::ChatCompletionUserMessageParam {
            content: $crate::content!($content),
            name: Some(check_name($name)),
        })
    }};
}

#[macro_export]
macro_rules! tool {
    ($tool_call_id:tt, $content:tt) => {{
        fn check_tool_call_id<T: AsRef<str>>(tool_call_id: T) -> String {
            tool_call_id.as_ref().to_string()
        }
        $crate::ChatCompletionMessageParam::Tool($crate::ChatCompletionToolMessageParam {
            tool_call_id: check_tool_call_id($tool_call_id),
            content: $crate::content!($content),
        })
    }};
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_content_macro() {
        // object
        let obj = content!({"key": "value"});
        if let Content::Object(val) = obj {
            assert!(val.is_object());
        } else {
            panic!("Expected Content::Object");
        }

        // string
        let str = content!("hello");
        if let Content::Text(val) = str {
            assert_eq!(val, "hello");
        } else {
            panic!("Expected Content::Text");
        }

        // array
        let arr = content!([1, 2, 3]);
        if let Content::Object(val) = arr {
            assert!(val.is_array());
        } else {
            panic!("Expected Content::Object");
        }

        // number
        let num = content!(42);
        if let Content::Text(val) = num {
            assert_eq!(val, "42");
        } else {
            panic!("Expected Content::Text");
        }

        // bool
        let bool_true = content!(true);
        if let Content::Text(val) = bool_true {
            assert_eq!(val, "true");
        } else {
            panic!("Expected Content::Text");
        }

        let bool_false = content!(false);
        if let Content::Text(val) = bool_false {
            assert_eq!(val, "false");
        } else {
            panic!("Expected Content::Text");
        }

        // null
        let null_val = content!(null);
        if let Content::Text(val) = null_val {
            assert_eq!(val, "null");
        } else {
            panic!("Expected Content::Text");
        }
    }

    #[test]
    fn test_system_macro() {
        let msg = system!("system message");
        if let ChatCompletionMessageParam::System(sys_msg) = msg {
            assert!(matches!(sys_msg.name, None));
            if let Content::Text(content) = sys_msg.content {
                assert_eq!(content, "system message");
            } else {
                panic!("Expected Content::Text");
            }
        } else {
            panic!("Expected ChatCompletionMessageParam::System");
        }

        let msg_with_name = system!("system message", "name");
        if let ChatCompletionMessageParam::System(sys_msg) = msg_with_name {
            assert!(matches!(sys_msg.name, Some(ref name) if name == "name"));
            if let Content::Text(content) = sys_msg.content {
                assert_eq!(content, "system message");
            } else {
                panic!("Expected Content::Text");
            }
        } else {
            panic!("Expected ChatCompletionMessageParam::System");
        }
    }

    #[test]
    fn test_user_macro() {
        let msg = user!("user message");
        if let ChatCompletionMessageParam::User(user_msg) = msg {
            assert!(matches!(user_msg.name, None));
            if let Content::Text(content) = user_msg.content {
                assert_eq!(content, "user message");
            } else {
                panic!("Expected Content::Text");
            }
        } else {
            panic!("Expected ChatCompletionMessageParam::User");
        }

        let msg_with_name = user!("user message", "name");
        if let ChatCompletionMessageParam::User(user_msg) = msg_with_name {
            assert!(matches!(user_msg.name, Some(ref name) if name == "name"));
            if let Content::Text(content) = user_msg.content {
                assert_eq!(content, "user message");
            } else {
                panic!("Expected Content::Text");
            }
        } else {
            panic!("Expected ChatCompletionMessageParam::User");
        }
    }

    #[test]
    fn test_tool_macro() {
        let tool_msg = tool!("call_123", {"result": 42});
        match tool_msg {
            ChatCompletionMessageParam::Tool(tool_msg_param) => {
                assert_eq!(tool_msg_param.tool_call_id, "call_123");

                if let Content::Object(val) = tool_msg_param.content {
                    assert!(val.is_object());
                    let obj = val.as_object().unwrap();
                    assert_eq!(
                        obj.get("result").and_then(|v| v.as_number()),
                        Some(&serde_json::Number::from(42))
                    );
                } else {
                    panic!("Expected Content::Object");
                }
            }
            _ => {
                panic!("Expected ChatCompletionMessageParam::Tool");
            }
        }
    }
}
