use bson::{doc, oid::ObjectId};

use crate::authentication::identity::Identity;
use crate::error::{AppError, AppResult};
use crate::models::{
    Booking, CreateVehicleRequest, UpdateVehicleRequest, Vehicle, VehicleFilters,
    VehiclePagination, VehicleQueryBuilder,
};
use crate::services;
use crate::validator;

/// Create a new vehicle (Admin only)
pub async fn create(identity: &Identity, request: CreateVehicleRequest) -> AppResult<Vehicle> {
    validator::vehicle::validate_brand_model(&request.brand, &request.metadata)
        .await
        .map_err(|e| AppError::bad_request(&e))?;

    let mut vehicle =
        Vehicle::new(request, identity.user_id.clone()).map_err(|e| AppError::bad_request(&e))?;

    let inserted_id = services::mongodb::insert_one(&vehicle, None).await?;
    vehicle.id = Some(inserted_id);

    Ok(vehicle)
}

/// Get vehicles with filters and pagination (All users)
pub async fn list(
    filters: VehicleFilters,
    pagination: VehiclePagination,
) -> AppResult<Vec<Vehicle>> {
    let query_builder = VehicleQueryBuilder {
        filters: Some(filters),
        pagination: Some(pagination),
    };

    let (filter, options) = query_builder.build_query();

    let vehicles = services::mongodb::collect_many(filter, options).await?;

    Ok(vehicles)
}

/// Update a vehicle (Admin, CarManager, MotorbikeManager)
pub async fn update(
    identity: &Identity,
    vehicle_id: &ObjectId,
    request: UpdateVehicleRequest,
) -> AppResult<Vehicle> {
    let filter = doc! { "_id": vehicle_id };

    let mut vehicle: Vehicle = services::mongodb::get_one(filter.clone(), None)
        .await?
        .ok_or_else(|| AppError::not_found("Vehicle not found"))?;
    validator::vehicle::validate_update_vehicle(identity, &vehicle, &request)?;

    // Update the vehicle (only description and price allowed)
    if let Some(description) = request.description {
        vehicle.description = Some(description);
    }
    if let Some(price_by_day) = request.price_by_day {
        vehicle.price_by_day = price_by_day;
    }

    // Save the updated vehicle using find_one_and_replace
    services::mongodb::find_one_and_replace(filter, &vehicle, None)
        .await?
        .ok_or_else(|| AppError::internal_server_error(format!("Failed to update vehicle")))?;

    Ok(vehicle)
}

/// Get a single vehicle by ID (All users)
pub async fn get(vehicle_id: &ObjectId) -> AppResult<Option<Vehicle>> {
    let filter = doc! { "_id": vehicle_id };

    let vehicle = services::mongodb::get_one(filter, None).await?;

    Ok(vehicle)
}

/// Get bookings for a specific vehicle (Admin, CarManager, MotorbikeManager)
pub async fn list_bookings(identity: &Identity, vehicle_id: &ObjectId) -> AppResult<Vec<Booking>> {
    let vehicle_filter = doc! { "_id": vehicle_id };
    let vehicle: Vehicle = services::mongodb::get_one(vehicle_filter, None)
        .await?
        .ok_or_else(|| AppError::not_found("Vehicle not found"))?;

    validator::vehicle::check_vehicle_type_permission(identity, &vehicle)?;
    let booking_filter = doc! { "vehicle_id": vehicle_id };
    let bookings = services::mongodb::collect_many(booking_filter, None).await?;

    Ok(bookings)
}
