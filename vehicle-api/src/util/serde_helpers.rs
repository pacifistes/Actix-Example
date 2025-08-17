use serde::{Deserialize, Deserializer};
use std::fmt::Display;
use std::str::FromStr;

/// Deserialize a comma-separated string into a Vec<T>
pub fn deserialize_comma_separated<'de, D, T>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: Display,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    match s {
        Some(s) if !s.trim().is_empty() => {
            let items: Result<Vec<T>, _> = s
                .split(',')
                .map(|item| item.trim())
                .filter(|item| !item.is_empty())
                .map(|item| item.parse::<T>())
                .collect();

            items
                .map(Some)
                .map_err(|e| serde::de::Error::custom(format!("Failed to parse item: {}", e)))
        }
        _ => Ok(None),
    }
}

/// Parse sort fields with optional +/- prefix
/// Example: "field1,-field2,+field3" -> [("field1", 1), ("field2", -1), ("field3", 1)]
pub fn parse_sort_fields(sort_str: &str) -> Vec<(String, i32)> {
    sort_str
        .split(',')
        .map(|field| field.trim())
        .filter(|field| !field.is_empty())
        .map(|field| {
            if field.starts_with('-') {
                (field[1..].to_string(), -1)
            } else if field.starts_with('+') {
                (field[1..].to_string(), 1)
            } else {
                (field.to_string(), 1)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[derive(serde::Deserialize)]
    struct TestStruct {
        #[serde(deserialize_with = "deserialize_comma_separated")]
        values: Option<Vec<String>>,
    }

    #[derive(serde::Deserialize)]
    struct TestBrandStruct {
        #[serde(deserialize_with = "deserialize_comma_separated")]
        brands: Option<Vec<crate::models::Brand>>,
    }

    #[test]
    fn test_comma_separated_deserialization() {
        let json = r#"{"values": "a,b,c"}"#;
        let parsed: TestStruct = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed.values,
            Some(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
    }

    #[test]
    fn test_comma_separated_brand_deserialization() {
        let json = r#"{"brands": "TESLA,MERCEDES"}"#;
        let parsed: TestBrandStruct = serde_json::from_str(json).unwrap();
        assert_eq!(
            parsed.brands,
            Some(vec![
                crate::models::Brand::TESLA,
                crate::models::Brand::MERCEDES
            ])
        );
    }

    #[test]
    fn test_sort_fields_parsing() {
        let result = parse_sort_fields("field1,-field2,+field3");
        assert_eq!(
            result,
            vec![
                ("field1".to_string(), 1),
                ("field2".to_string(), -1),
                ("field3".to_string(), 1),
            ]
        );
    }
}
