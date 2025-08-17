use bson::{doc, oid::ObjectId};
use chrono::NaiveDate;

use crate::error::AppResult;
use crate::services;

/// Check if there are any overlapping bookings for a specific vehicle and date range
/// Only considers bookings with PENDING or CONFIRMED status as conflicts
pub async fn has_overlapping_bookings(
    vehicle_id: ObjectId,
    from_date: NaiveDate,
    to_date: NaiveDate,
) -> AppResult<bool> {
    // Build the overlap query with status filtering
    let from_bson = bson::to_bson(&from_date).map_err(|e| {
        crate::error::AppError::internal_server_error(format!("BSON conversion error: {}", e))
    })?;
    let to_bson = bson::to_bson(&to_date).map_err(|e| {
        crate::error::AppError::internal_server_error(format!("BSON conversion error: {}", e))
    })?;

    // Find overlapping bookings that are PENDING or CONFIRMED
    let filter = doc! {
        "vehicle_id": vehicle_id,
        "$and": [
            { "from_date": { "$lte": to_bson } },      // existing.start <= new.end
            { "to_date": { "$gte": from_bson } },      // existing.end >= new.start
            { "$or": [
                { "status": "PENDING" },
                { "status": "CONFIRMED" }
            ]}
        ]
    };

    let bookings: Vec<crate::models::Booking> =
        services::mongodb::collect_many(filter, None).await?;

    Ok(!bookings.is_empty())
}
