use bson::oid::ObjectId;
use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
use validator::Validate;

// =============================================================================
// ENUMS
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, EnumString, Display, PartialEq)]
#[serde(tag = "status", content = "reason", rename_all = "UPPERCASE")]
#[strum(serialize_all = "UPPERCASE")]
pub enum BookingStatus {
    Pending,
    Confirmed,
    Rejected(String),
    Cancelled(String),
}

// =============================================================================
// MAIN BOOKING STRUCT
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Booking {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub vehicle_id: ObjectId,
    pub customer_id: String, // User ID of the customer who made the booking
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
    #[serde(flatten)]
    pub status: BookingStatus,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub order_date: DateTime<Utc>, // When the booking was created
}

// =============================================================================
// REQUEST/RESPONSE STRUCTS
// =============================================================================

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct CreateBookingRequest {
    pub vehicle_id: ObjectId,
    pub from_date: NaiveDate,
    pub to_date: NaiveDate,
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct UpdateBookingRequest {
    pub status: Option<BookingStatus>,
}

// =============================================================================
// IMPLEMENTATIONS - CORE BOOKING METHODS
// =============================================================================

impl crate::services::mongodb::MongoStruct for Booking {
    fn get_collection() -> &'static str {
        "bookings"
    }
}

impl Booking {
    pub fn new(request: CreateBookingRequest, customer_id: String) -> Self {
        Self {
            id: None,
            vehicle_id: request.vehicle_id,
            customer_id,
            from_date: request.from_date,
            to_date: request.to_date,
            status: BookingStatus::Pending,
            order_date: Utc::now(),
        }
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_booking_status_serialization() {
        let pending = BookingStatus::Pending;
        let cancelled = BookingStatus::Cancelled("User request".to_string());
        let rejected = BookingStatus::Rejected("Invalid dates".to_string());

        // Test that serialization works and uses UPPERCASE
        let pending_json = serde_json::to_string(&pending).unwrap();
        let cancelled_json = serde_json::to_string(&cancelled).unwrap();
        let rejected_json = serde_json::to_string(&rejected).unwrap();

        // Verify uppercase status values
        assert!(pending_json.contains("\"status\":\"PENDING\""));
        assert!(cancelled_json.contains("\"status\":\"CANCELLED\""));
        assert!(cancelled_json.contains("\"reason\":\"User request\""));
        assert!(rejected_json.contains("\"status\":\"REJECTED\""));
        assert!(rejected_json.contains("\"reason\":\"Invalid dates\""));
    }
}
