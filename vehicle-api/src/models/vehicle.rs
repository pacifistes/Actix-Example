use bson::oid::ObjectId;
use bson::Document;
use chrono::{DateTime, Utc};
use derive_builder::Builder;
use macros::CustomValidate;
use mongodb::options::FindOptions;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use validator::Validate;

use crate::authentication::identity::Identity;
use crate::services;
use crate::util::serde_helpers::parse_sort_fields;
use crate::validator::CustomValidateTrait;

// =============================================================================
// ENUMS
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, EnumString, Display, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum VehicleType {
    CAR,
    MOTORBIKE,
}

#[derive(Clone, Debug, Serialize, Deserialize, EnumString, Display, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Gearbox {
    MANUAL,
    AUTOMATIC,
}

#[derive(Clone, Debug, Serialize, Deserialize, EnumString, Display, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum FuelType {
    PETROL,
    DIESEL,
    ELECTRIC,
}

#[derive(Clone, Debug, Serialize, Deserialize, EnumString, Display, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Brand {
    TESLA,
    MERCEDES,
}

#[derive(Clone, Debug, Serialize, Deserialize, EnumString, Display, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum CarModel {
    // Tesla models
    MODEL_S,
    MODEL_3,
    MODEL_X,
    MODEL_Y,
    CYBERTRUCK,
    ROADSTER,
    // Mercedes models
    A_CLASS,
    C_CLASS,
    E_CLASS,
    S_CLASS,
    G_CLASS,
    GLC,
    GLE,
    AMG_GT,
}

#[derive(Clone, Debug, Serialize, Deserialize, EnumString, Display, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MotorbikeModel {
    SPORTBIKE,
    CRUISER,
}

// =============================================================================
// METADATA STRUCTS
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CarMetadata {
    pub brand: Brand,
    pub model: CarModel,
    pub seats: u8,
    pub fuel_type: FuelType,
    pub gearbox: Gearbox,
    pub engine_cc: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MotorbikeMetadata {
    pub brand: Brand,
    pub model: MotorbikeModel,
    pub engine_cc: u32,
    pub has_sidecar: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "metadata")]
pub enum VehicleMetadata {
    Car(CarMetadata),
    Motorbike(MotorbikeMetadata),
}

// =============================================================================
// MAIN VEHICLE STRUCT
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vehicle {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub brand: Brand,
    #[serde(flatten)]
    pub metadata: VehicleMetadata,
    pub description: Option<String>,
    pub price_by_day: f64,
    pub year_of_production: u32,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub added_at: DateTime<Utc>,
    pub added_by: String,
}

// =============================================================================
// REQUEST/RESPONSE STRUCTS
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, Validate, CustomValidate)]
pub struct CreateVehicleRequest {
    pub brand: Brand,
    #[custom_validate(custom(function = "crate::validator::vehicle::validate_metadata"))]
    pub metadata: VehicleMetadata,
    #[validate(length(
        min = 1,
        max = 249,
        message = "Description must be between 1 and 249 characters"
    ))]
    pub description: Option<String>,
    #[validate(range(min = 0.01, message = "Price must be greater than 0"))]
    pub price_by_day: f64,
    #[validate(range(min = 1900, max = 2030, message = "Year must be between 1900 and 2030"))]
    pub year_of_production: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct UpdateVehicleRequest {
    #[validate(length(
        min = 1,
        max = 249,
        message = "Description must be between 1 and 249 characters"
    ))]
    pub description: Option<Option<String>>, // Option<Option<String>> to allow clearing
    #[validate(range(min = 0.01, message = "Price must be greater than 0"))]
    pub price_by_day: Option<f64>,
}

// =============================================================================
// FILTERING AND PAGINATION STRUCTS
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct VehicleFilters {
    // Brand and model filters (comma-separated, using enum types)
    #[serde(
        deserialize_with = "crate::util::serde_helpers::deserialize_comma_separated",
        default
    )]
    pub brand: Option<Vec<Brand>>,

    // Model can be either CarModel or MotorbikeModel, so we keep it as String for flexibility
    #[serde(
        deserialize_with = "crate::util::serde_helpers::deserialize_comma_separated",
        default
    )]
    pub model: Option<Vec<String>>,

    // Price filters (min and max separately)
    pub min_price: Option<f64>,
    pub max_price: Option<f64>,

    // Year filters (comma-separated)
    #[serde(
        deserialize_with = "crate::util::serde_helpers::deserialize_comma_separated",
        default
    )]
    pub year_of_production: Option<Vec<u32>>,

    // Vehicle metadata filters (using enum types)
    #[serde(
        deserialize_with = "crate::util::serde_helpers::deserialize_comma_separated",
        default
    )]
    pub fuel_type: Option<Vec<FuelType>>,

    #[serde(
        deserialize_with = "crate::util::serde_helpers::deserialize_comma_separated",
        default
    )]
    pub gearbox: Option<Vec<Gearbox>>,

    #[serde(
        deserialize_with = "crate::util::serde_helpers::deserialize_comma_separated",
        default
    )]
    pub seats: Option<Vec<u8>>,

    #[serde(
        deserialize_with = "crate::util::serde_helpers::deserialize_comma_separated",
        default
    )]
    pub engine_cc: Option<Vec<u32>>,

    // Boolean filter for motorbike sidecar
    pub has_sidecar: Option<bool>,

    // Date range filters (for added_at field)
    pub added_at_from: Option<DateTime<Utc>>,
    pub added_at_to: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VehiclePagination {
    pub page: Option<i64>,
    pub limit: Option<i64>,
    pub sort: Option<String>,
}

#[derive(Builder, Clone, Debug, Default)]
#[builder(setter(into, strip_option), default)]
pub struct VehicleQueryBuilder {
    pub filters: Option<VehicleFilters>,
    pub pagination: Option<VehiclePagination>,
}

// =============================================================================
// IMPLEMENTATIONS - CORE VEHICLE METHODS
// =============================================================================

impl crate::services::mongodb::MongoStruct for Vehicle {
    fn get_collection() -> &'static str {
        "vehicles"
    }
}

impl Vehicle {
    pub fn new(request: CreateVehicleRequest, added_by: String) -> Result<Self, String> {
        Ok(Self {
            id: None,
            brand: request.brand,
            metadata: request.metadata,
            description: request.description,
            price_by_day: request.price_by_day,
            year_of_production: request.year_of_production,
            added_at: Utc::now(),
            added_by,
        })
    }
}

// =============================================================================
// IMPLEMENTATIONS - QUERY BUILDING
// =============================================================================

impl VehicleFilters {
    /// Convert filters to BSON document for MongoDB query (Ultra-clean version using QueryBuilder)
    pub fn to_bson_filter(&self) -> Document {
        let mut filter = Document::new();
        let builder = services::mongodb::QueryBuilder::new();

        // String-based enum filters
        builder.add_string_filter(&mut filter, "brand", &self.brand);
        builder.add_string_filter(&mut filter, "metadata.fuel_type", &self.fuel_type);
        builder.add_string_filter(&mut filter, "metadata.gearbox", &self.gearbox);

        // Direct value filters
        builder.add_filter(&mut filter, "year_of_production", &self.year_of_production);
        // Convert u8 to i32 for BSON compatibility
        if let Some(seats) = &self.seats {
            let seats_i32: Option<Vec<i32>> = Some(seats.iter().map(|&s| s as i32).collect());
            builder.add_filter(&mut filter, "metadata.seats", &seats_i32);
        }
        builder.add_filter(&mut filter, "metadata.engine_cc", &self.engine_cc);
        builder.add_filter(&mut filter, "metadata.model", &self.model);

        // Boolean filter
        builder.add_boolean_filter(&mut filter, "metadata.has_sidecar", self.has_sidecar);

        // Price range filter using min/max
        builder.add_range_filter(&mut filter, "price_by_day", self.min_price, self.max_price);

        // Date range filter using the range method
        builder.add_range_filter(
            &mut filter,
            "added_at",
            self.added_at_from,
            self.added_at_to,
        );

        filter
    }
}

impl VehiclePagination {
    /// Convert pagination to MongoDB FindOptions
    pub fn to_find_options(&self) -> FindOptions {
        let mut options = FindOptions::default();

        // Set limit
        if let Some(limit) = self.limit {
            options.limit = Some(limit);
        }

        // Set skip for pagination
        if let Some(page) = self.page {
            let limit = self.limit.unwrap_or(10);
            let skip = (page - 1) * limit;
            if skip > 0 {
                options.skip = Some(skip as u64);
            }
        }

        // Set sort
        if let Some(sort_str) = &self.sort {
            let sort_fields = parse_sort_fields(sort_str);
            if !sort_fields.is_empty() {
                let mut sort_doc = Document::new();
                for (field, direction) in sort_fields {
                    sort_doc.insert(field, direction);
                }
                options.sort = Some(sort_doc);
            }
        }

        options
    }
}

impl VehicleQueryBuilder {
    pub fn build_query(&self) -> (Document, FindOptions) {
        let filter = self
            .filters
            .as_ref()
            .map(|f| f.to_bson_filter())
            .unwrap_or_else(|| Document::new());

        let options = self
            .pagination
            .as_ref()
            .map(|p| p.to_find_options())
            .unwrap_or_else(|| FindOptions::builder().build());

        (filter, options)
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::VehicleFilters;

    #[test]
    fn test_brand_filter_single() {
        use crate::models::Brand;
        let filters = VehicleFilters {
            brand: Some(vec![Brand::TESLA]),
            ..Default::default()
        };
        let doc = filters.to_bson_filter();
        assert_eq!(doc.get_str("brand").unwrap(), "TESLA");
    }

    #[test]
    fn test_brand_filter_multiple() {
        use crate::models::Brand;
        let filters = VehicleFilters {
            brand: Some(vec![Brand::TESLA, Brand::MERCEDES]),
            ..Default::default()
        };
        let doc = filters.to_bson_filter();
        assert!(doc.contains_key("brand"));
    }

    #[test]
    fn test_sort_fields_parsing() {
        let pagination = VehiclePagination {
            page: Some(1),
            limit: Some(10),
            sort: Some("price_by_day,-year_of_production,+brand".to_string()),
        };
        let options = pagination.to_find_options();
        // Test that sort document is created correctly
        assert!(options.sort.is_some());
    }
}
