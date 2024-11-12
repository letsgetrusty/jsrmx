use serde_json::Value;

pub enum JsonField {
    String(String),
    Value(Value),
}

impl JsonField {
    pub fn unescape(self) -> Value {
        match self {
            Self::String(string) => {
                serde_json::from_str(&string.replace("\\\"", "\"")).unwrap_or(Value::Null)
            }
            Self::Value(json) => json,
        }
    }

    pub fn escape(self) -> Value {
        match self {
            Self::String(string) => Value::String(string),
            Self::Value(json) => Value::String(json.to_string()),
        }
    }
}

impl From<String> for JsonField {
    fn from(string: String) -> Self {
        Self::String(string)
    }
}

impl From<Value> for JsonField {
    fn from(json: Value) -> Self {
        match json {
            Value::String(string) => Self::String(string),
            _ => Self::Value(json),
        }
    }
}
