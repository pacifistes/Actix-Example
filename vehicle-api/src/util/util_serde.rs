use mongodb::bson::{self, Bson};
use serde::Serialize;
use serde_json::{json, Value};

fn bson_to_value(value: Bson) -> Value {
    match value {
        Bson::Int32(i) => i.into(),
        Bson::Int64(i) => i.into(),
        Bson::Double(f) if f.is_nan() => {
            let s = if f.is_sign_negative() { "-NaN" } else { "NaN" };
            json!(s)
        }
        Bson::Double(f) if f.is_infinite() => {
            let s = if f.is_sign_negative() {
                "-Infinity"
            } else {
                "Infinity"
            };
            json!(s)
        }
        Bson::Double(f) => json!((f * 100.0).round() / 100.0),
        Bson::String(s) => json!(s),
        Bson::Array(arr) => Value::Array(arr.into_iter().map(bson_to_value).collect()),
        Bson::Document(arr) => Value::Object(
            arr.into_iter()
                .map(|(k, v)| {
                    if k.as_str() == "_id" {
                        (String::from("id"), bson_to_value(v))
                    } else {
                        (k, bson_to_value(v))
                    }
                })
                .collect(),
        ),
        Bson::Boolean(v) => json!(v),
        Bson::Null => Value::Null,
        Bson::ObjectId(oid) => json!(oid.to_hex()),
        Bson::DateTime(date)
            if date.timestamp_millis() >= 0
                && chrono::Datelike::year(&date.to_chrono()) <= 99999 =>
        {
            json!(date
                .to_chrono()
                .to_rfc3339_opts(chrono::SecondsFormat::AutoSi, true))
        }
        Bson::DateTime(date) => json!(date.timestamp_millis().to_string()),
        _ => {
            let context = format!("Attempted conversion of invalid data type: {value}");
            panic!("{context}");
        }
    }
}

pub fn to_value<T>(value: T) -> Value
where
    T: Serialize,
{
    let bson = bson::to_bson(&value).unwrap();
    bson_to_value(bson)
}
