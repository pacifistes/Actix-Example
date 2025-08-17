use crate::authentication::identity::{Identity, Role};
use crate::error::{AppError, AppResult};
use crate::models::{Booking, BookingStatus, CreateBookingRequest, UpdateBookingRequest};
use crate::services::mongodb::booking;

/// Validate booking creation request
/// Checks date range and vehicle availability (overlap conflicts)
pub async fn validate_booking_creation(
    _identity: &Identity,
    request: &CreateBookingRequest,
) -> Result<(), String> {
    // Validate date range
    if request.from_date >= request.to_date {
        return Err("from_date must be before to_date".to_string());
    }

    // Check for overlapping bookings
    match booking::has_overlapping_bookings(request.vehicle_id, request.from_date, request.to_date)
        .await
    {
        Ok(has_overlap) => {
            if has_overlap {
                return Err("Vehicle is already booked for overlapping dates.".to_string());
            }
        }
        Err(_) => {
            return Err("Failed to check for booking conflicts.".to_string());
        }
    }

    Ok(())
}

/// Check if user has permission to update this booking and validate the update
pub fn validate_update_booking(
    identity: &Identity,
    booking: &Booking,
    request: &UpdateBookingRequest,
) -> AppResult<()> {
    // Check general update permission
    check_booking_update_permission(identity, booking)?;

    // If status change is requested, validate it based on role
    if let Some(ref new_status) = request.status {
        match identity.role {
            Role::Customer => {
                validate_customer_status_change(&booking.status, new_status)?;
            }
            Role::Admin | Role::CarManager | Role::MotorbikeManager => {
                validate_non_customer_status_change(&booking.status, new_status)?;
            }
        }
    }

    Ok(())
}

/// Check if user has permission to update this booking
pub fn check_booking_update_permission(identity: &Identity, booking: &Booking) -> AppResult<()> {
    match identity.role {
        Role::Admin => Ok(()), // Admin can update any booking
        Role::CarManager | Role::MotorbikeManager => Ok(()), // Managers can update any booking
        Role::Customer => {
            // Customers can only update their own bookings
            if booking.customer_id == identity.user_id {
                Ok(())
            } else {
                Err(AppError::forbidden(
                    "You can only update your own bookings.",
                ))
            }
        }
    }
}

/// Check if user has permission to view this booking
pub fn check_booking_view_permission(identity: &Identity, booking: &Booking) -> AppResult<()> {
    match identity.role {
        Role::Admin | Role::CarManager | Role::MotorbikeManager => {
            // Managers and Admin can see all bookings
            Ok(())
        }
        Role::Customer => {
            // Customers can only see their own bookings
            if booking.customer_id == identity.user_id {
                Ok(())
            } else {
                Err(AppError::forbidden("You can only view your own bookings."))
            }
        }
    }
}

/// Validate that customers can only cancel bookings if status is PENDING or CONFIRMED
fn validate_customer_status_change(
    current_status: &BookingStatus,
    new_status: &BookingStatus,
) -> AppResult<()> {
    match new_status {
        BookingStatus::Cancelled(_) => {
            // Customers can cancel only if current status is PENDING or CONFIRMED
            match current_status {
                BookingStatus::Pending | BookingStatus::Confirmed => Ok(()),
                _ => Err(AppError::forbidden(
                    "You can only cancel bookings that are pending or confirmed.",
                )),
            }
        }
        _ => {
            // Customers can only cancel bookings, not change to other statuses
            Err(AppError::forbidden(
                "Customers can only cancel their bookings.",
            ))
        }
    }
}

/// Validate that non-customers (Admin, Managers) can only use CONFIRMED/REJECTED statuses
fn validate_non_customer_status_change(
    current_status: &BookingStatus,
    new_status: &BookingStatus,
) -> AppResult<()> {
    match new_status {
        BookingStatus::Cancelled(_) => Err(AppError::forbidden(
            "Only customers can cancel bookings. Use reject status instead.",
        )),
        BookingStatus::Pending => Err(AppError::forbidden("Cannot change status back to pending.")),
        BookingStatus::Confirmed | BookingStatus::Rejected(_) => {
            // Additional business logic for admin/manager status changes
            match current_status {
                BookingStatus::Pending => Ok(()), // Can confirm or reject pending bookings
                BookingStatus::Confirmed => {
                    // Can only reject confirmed bookings
                    match new_status {
                        BookingStatus::Rejected(_) => Ok(()),
                        _ => Err(AppError::bad_request(
                            "Confirmed bookings can only be rejected.",
                        )),
                    }
                }
                BookingStatus::Rejected(_) | BookingStatus::Cancelled(_) => Err(
                    AppError::bad_request("Cannot modify rejected or cancelled bookings."),
                ),
            }
        }
    }
}
