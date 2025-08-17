use actix_web::web::ReqData;
use actix_web::{get, patch, post, web, HttpResponse, Result};
use actix_web_grants::proc_macro::protect;
use bson::oid::ObjectId;

use crate::authentication::identity::Identity;
use crate::error::AppError;
use crate::models::{
    CreateVehicleRequest, UpdateVehicleRequest, VehicleFilters, VehiclePagination,
};
use crate::validator;
use crate::{controllers, util};

/// POST /vehicles - Create a new vehicle (Admin only)
#[post("/vehicles")]
#[protect("Admin")]
async fn create(
    identity: ReqData<Identity>,
    request: validator::Json<CreateVehicleRequest>,
) -> Result<HttpResponse, AppError> {
    let result = controllers::vehicle::create(&identity, request.into_inner()).await;

    match result {
        Ok(vehicle) => Ok(HttpResponse::Created().json(util::util_serde::to_value(vehicle))),
        Err(error) => Err(error),
    }
}

/// GET /vehicles - List vehicles with filters and pagination (All users)
#[get("/vehicles")]
async fn list(
    _identity: ReqData<Identity>,
    web::Query(filters): web::Query<VehicleFilters>,
    web::Query(pagination): web::Query<VehiclePagination>,
) -> Result<HttpResponse, AppError> {
    let result = controllers::vehicle::list(filters, pagination).await;

    match result {
        Ok(vehicles) => Ok(HttpResponse::Ok().json(util::util_serde::to_value(vehicles))),
        Err(error) => Err(error),
    }
}

/// PATCH /vehicles/{vehicle_id} - Update a vehicle (Admin, CarManager, MotorbikeManager)
#[patch("/vehicles/{vehicle_id}")]
#[protect("Admin", "CarManager", "MotorbikeManager")]
async fn update(
    identity: ReqData<Identity>,
    path: web::Path<String>,
    web::Json(request): web::Json<UpdateVehicleRequest>,
) -> Result<HttpResponse, AppError> {
    let vehicle_id = ObjectId::parse_str(&path.into_inner())
        .map_err(|_| AppError::bad_request("Invalid vehicle ID format"))?;

    let result = controllers::vehicle::update(&identity, &vehicle_id, request).await;

    match result {
        Ok(vehicle) => Ok(HttpResponse::Ok().json(util::util_serde::to_value(vehicle))),
        Err(error) => Err(error),
    }
}

/// GET /vehicles/{vehicle_id} - Get a single vehicle (All users)
#[get("/vehicles/{vehicle_id}")]
async fn get(
    _identity: ReqData<Identity>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let vehicle_id = ObjectId::parse_str(&path.into_inner())
        .map_err(|_| AppError::bad_request("Invalid vehicle ID format"))?;

    let result = controllers::vehicle::get(&vehicle_id).await;

    match result {
        Ok(Some(vehicle)) => Ok(HttpResponse::Ok().json(util::util_serde::to_value(vehicle))),
        Ok(None) => Err(AppError::not_found("Vehicle not found")),
        Err(error) => Err(error),
    }
}

/// GET /vehicles/{vehicle_id}/bookings - Get all bookings for a vehicle (Admin, CarManager, MotorbikeManager)
#[get("/vehicles/{vehicle_id}/bookings")]
#[protect("Admin", "CarManager", "MotorbikeManager")]
async fn list_bookings(
    identity: ReqData<Identity>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let vehicle_id_str = path.into_inner();
    let vehicle_id = ObjectId::parse_str(&vehicle_id_str)
        .map_err(|_| AppError::bad_request("Invalid vehicle ID format"))?;
    let result = controllers::vehicle::list_bookings(&identity, &vehicle_id).await;

    match result {
        Ok(bookings) => Ok(HttpResponse::Ok().json(util::util_serde::to_value(bookings))),
        Err(error) => Err(error),
    }
}

pub fn configure(config: &mut web::ServiceConfig) {
    config
        .service(create)
        .service(list)
        .service(update)
        .service(get)
        .service(list_bookings);
}
