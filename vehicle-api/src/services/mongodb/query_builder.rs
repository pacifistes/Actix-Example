use bson::{doc, Bson, Document};
use std::fmt::Display;

/// Generic MongoDB query builder for filters
pub struct QueryBuilder;

impl QueryBuilder {
    /// Create a new query builder instance
    pub fn new() -> Self {
        Self
    }

    /// Add a filter for types that can convert to BSON (Equal strategy only)
    pub fn add_filter<T>(&self, filter: &mut Document, field: &str, values: &Option<Vec<T>>)
    where
        T: Clone + Into<Bson>,
    {
        if let Some(vals) = values {
            if vals.is_empty() {
                return;
            }

            match vals.len() {
                1 => {
                    filter.insert(field, vals[0].clone().into());
                }
                _ => {
                    let bson_vals: Vec<Bson> = vals.iter().map(|v| v.clone().into()).collect();
                    filter.insert(field, doc! { "$in": bson_vals });
                }
            }
        }
    }

    /// Add a string-based filter for types that implement Display (like enums)
    pub fn add_string_filter<T>(&self, filter: &mut Document, field: &str, values: &Option<Vec<T>>)
    where
        T: Display,
    {
        if let Some(vals) = values {
            if vals.is_empty() {
                return;
            }

            match vals.len() {
                1 => {
                    filter.insert(field, vals[0].to_string());
                }
                _ => {
                    let string_vals: Vec<String> = vals.iter().map(|v| v.to_string()).collect();
                    filter.insert(field, doc! { "$in": string_vals });
                }
            }
        }
    }

    /// Range filter that supports min-only, max-only, or min-max
    /// Usage: add_range_filter(filter, "price", Some(100.0), Some(500.0))
    ///        add_range_filter(filter, "price", Some(100.0), None) // min only
    ///        add_range_filter(filter, "price", None, Some(500.0)) // max only
    pub fn add_range_filter<T>(
        &self,
        filter: &mut Document,
        field: &str,
        min_val: Option<T>,
        max_val: Option<T>,
    ) where
        T: Clone + Into<Bson>,
    {
        if min_val.is_none() && max_val.is_none() {
            return;
        }

        let mut range_doc = Document::new();

        if let Some(min) = min_val {
            range_doc.insert("$gte", min.into());
        }

        if let Some(max) = max_val {
            range_doc.insert("$lte", max.into());
        }

        filter.insert(field, range_doc);
    }

    /// Add a boolean filter
    pub fn add_boolean_filter(&self, filter: &mut Document, field: &str, value: Option<bool>) {
        if let Some(val) = value {
            filter.insert(field, val);
        }
    }
}

impl Default for QueryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equal_filter_single_value() {
        let builder = QueryBuilder::new();
        let mut filter = Document::new();
        let values = Some(vec![42]);

        builder.add_filter(&mut filter, "test_field", &values);

        assert_eq!(filter.get_i32("test_field").unwrap(), 42);
    }

    #[test]
    fn test_equal_filter_multiple_values() {
        let builder = QueryBuilder::new();
        let mut filter = Document::new();
        let values = Some(vec![1, 2, 3]);

        builder.add_filter(&mut filter, "test_field", &values);

        assert!(filter.contains_key("test_field"));
        // Should create a $in filter
    }

    #[test]
    fn test_range_filter_min_only() {
        let builder = QueryBuilder::new();
        let mut filter = Document::new();

        builder.add_range_filter(&mut filter, "price", Some(100.0), None);

        let price_filter = filter.get_document("price").unwrap();
        assert_eq!(price_filter.get_f64("$gte").unwrap(), 100.0);
        assert!(!price_filter.contains_key("$lte"));
    }

    #[test]
    fn test_range_filter_max_only() {
        let builder = QueryBuilder::new();
        let mut filter = Document::new();

        builder.add_range_filter(&mut filter, "price", None, Some(500.0));

        let price_filter = filter.get_document("price").unwrap();
        assert_eq!(price_filter.get_f64("$lte").unwrap(), 500.0);
        assert!(!price_filter.contains_key("$gte"));
    }

    #[test]
    fn test_range_filter_min_max() {
        let builder = QueryBuilder::new();
        let mut filter = Document::new();

        builder.add_range_filter(&mut filter, "price", Some(100.0), Some(500.0));

        let price_filter = filter.get_document("price").unwrap();
        assert_eq!(price_filter.get_f64("$gte").unwrap(), 100.0);
        assert_eq!(price_filter.get_f64("$lte").unwrap(), 500.0);
    }

    #[test]
    fn test_string_filter() {
        let builder = QueryBuilder::new();
        let mut filter = Document::new();
        let values = Some(vec!["TESLA".to_string(), "MERCEDES".to_string()]);

        builder.add_string_filter(&mut filter, "brand", &values);

        assert!(filter.contains_key("brand"));
    }

    #[test]
    fn test_boolean_filter() {
        let builder = QueryBuilder::new();
        let mut filter = Document::new();

        builder.add_boolean_filter(&mut filter, "has_feature", Some(true));

        assert_eq!(filter.get_bool("has_feature").unwrap(), true);
    }
}
