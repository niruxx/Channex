//! Lenient field access for imageboard JSON, where the same field can be
//! a number on one engine and a numeric string (or a bool) on another.

use serde_json::Value;

pub fn str_of(v: &Value, key: &str) -> String {
    match v.get(key) {
        Some(Value::String(s)) => s.clone(),
        Some(Value::Number(n)) => n.to_string(),
        _ => String::new(),
    }
}

pub fn i64_of(v: &Value, key: &str) -> i64 {
    match v.get(key) {
        Some(Value::Number(n)) => n
            .as_i64()
            .or_else(|| n.as_f64().map(|f| f as i64))
            .unwrap_or(0),
        Some(Value::String(s)) => s.trim().parse().unwrap_or(0),
        Some(Value::Bool(b)) => i64::from(*b),
        _ => 0,
    }
}

pub fn u32_of(v: &Value, key: &str) -> u32 {
    u32::try_from(i64_of(v, key).max(0)).unwrap_or(u32::MAX)
}

pub fn u64_of(v: &Value, key: &str) -> u64 {
    u64::try_from(i64_of(v, key).max(0)).unwrap_or(0)
}

/// `true`, `1` and `"1"` are all true.
pub fn bool_of(v: &Value, key: &str) -> bool {
    match v.get(key) {
        Some(Value::Bool(b)) => *b,
        Some(_) => i64_of(v, key) != 0,
        None => false,
    }
}

pub fn has(v: &Value, key: &str) -> bool {
    v.get(key).is_some_and(|x| !x.is_null())
}
