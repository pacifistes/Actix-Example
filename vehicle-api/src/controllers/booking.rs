use bson::{doc, oid::ObjectId};

use crate::authentication::identity::Identity;
use crate::error::{AppError, AppResult};
use crate::models::{Booking, CreateBookingRequest, UpdateBookingRequest, Vehicle};
use crate::services;
use crate::validator;

/// Create a new booking (Customer)
pub async fn create(identity: &Identity, request: CreateBookingRequest) -> AppResult<Booking> {
    // Validate booking creation (date range and overlap checking)
    crate::validator::booking::validate_booking_creation(identity, &request)
        .await
        .map_err(|e| AppError::bad_request(&e))?;

    // Check if vehicle exists
    let vehicle_filter = doc! { "_id": request.vehicle_id };
    let _vehicle: Vehicle = services::mongodb::get_one(vehicle_filter, None)
        .await?
        .ok_or_else(|| AppError::not_found("Vehicle not found"))?;

    // Create the booking
    let mut booking = Booking::new(request, identity.user_id.clone());

    let inserted_id = services::mongodb::insert_one(&booking, None).await?;
    booking.id = Some(inserted_id);

    Ok(booking)
}

/// List bookings (simplified without filters and pagination)
pub async fn list(identity: &Identity) -> AppResult<Vec<Booking>> {
    let mut filter = bson::Document::new();

    // Apply permission-based filtering for customers
    if matches!(
        identity.role,
        crate::authentication::identity::Role::Customer
    ) {
        // Customers can only see their own bookings
        filter.insert("customer_id", &identity.user_id);
    }

    let bookings = services::mongodb::collect_many(filter, None).await?;

    Ok(bookings)
}

/// Update a booking (Admin, CarManager, MotorbikeManager, Customer - for their own bookings)
pub async fn update(
    identity: &Identity,
    booking_id: &ObjectId,
    request: UpdateBookingRequest,
) -> AppResult<Booking> {
    // Get the existing booking
    let filter = doc! { "_id": booking_id };

    let mut booking: Booking = services::mongodb::get_one(filter.clone(), None)
        .await?
        .ok_or_else(|| AppError::not_found("Booking not found"))?;

    // Validate the update (permissions and business rules)
    validator::booking::validate_update_booking(identity, &booking, &request)?;

    // Update the booking status
    if let Some(new_status) = request.status {
        booking.status = new_status;
    }

    // Save the updated booking
    let updated_booking = services::mongodb::find_one_and_replace(filter, booking, None)
        .await?
        .ok_or_else(|| AppError::internal_server_error("Failed to update booking"))?;

    Ok(updated_booking)
}

/// Get a single booking by ID
pub async fn get(identity: &Identity, booking_id: &ObjectId) -> AppResult<Option<Booking>> {
    let filter = doc! { "_id": booking_id };
    let booking: Option<Booking> = services::mongodb::get_one(filter, None).await?;

    // Check permissions for viewing this booking
    if let Some(ref booking) = booking {
        validator::booking::check_booking_view_permission(identity, booking)?;
    }

    Ok(booking)
}
