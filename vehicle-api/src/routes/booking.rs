use actix_web::web::ReqData;
use actix_web::{get, patch, post, web, HttpResponse, Result};
use actix_web_grants::proc_macro::protect;
use bson::oid::ObjectId;

use crate::authentication::identity::Identity;
use crate::error::AppError;
use crate::models::{CreateBookingRequest, UpdateBookingRequest};
use crate::{controllers, util};

/// POST /bookings - Create a new booking (Customer only)
#[post("/bookings")]
#[protect("Customer")]
async fn create(
    identity: ReqData<Identity>,
    web::Json(request): web::Json<CreateBookingRequest>,
) -> Result<HttpResponse, AppError> {
    let result = controllers::booking::create(&identity, request).await;

    match result {
        Ok(booking) => Ok(HttpResponse::Created().json(util::util_serde::to_value(booking))),
        Err(error) => Err(error),
    }
}

/// GET /bookings - List bookings (simplified)
/// Customer: only sees their own bookings
/// Admin/Managers: can view all bookings
#[get("/bookings")]
#[protect("Admin", "CarManager", "MotorbikeManager", "Customer")]
async fn list(identity: ReqData<Identity>) -> Result<HttpResponse, AppError> {
    let result = controllers::booking::list(&identity).await;

    match result {
        Ok(bookings) => Ok(HttpResponse::Ok().json(util::util_serde::to_value(bookings))),
        Err(error) => Err(error),
    }
}

/// PATCH /bookings/{booking_id} - Update a booking (Admin, CarManager, MotorbikeManager, Customer for own bookings)
#[patch("/bookings/{booking_id}")]
#[protect("Admin", "CarManager", "MotorbikeManager", "Customer")]
async fn update(
    identity: ReqData<Identity>,
    path: web::Path<String>,
    web::Json(request): web::Json<UpdateBookingRequest>,
) -> Result<HttpResponse, AppError> {
    let booking_id = ObjectId::parse_str(&path.into_inner())
        .map_err(|_| AppError::bad_request("Invalid booking ID format"))?;

    let result = controllers::booking::update(&identity, &booking_id, request).await;

    match result {
        Ok(booking) => Ok(HttpResponse::Ok().json(util::util_serde::to_value(booking))),
        Err(error) => Err(error),
    }
}

/// GET /bookings/{booking_id} - Get a single booking
/// Customer: only their own bookings
/// Admin/Managers: any booking
#[get("/bookings/{booking_id}")]
#[protect("Admin", "CarManager", "MotorbikeManager", "Customer")]
async fn get(
    identity: ReqData<Identity>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError> {
    let booking_id_str = path.into_inner();
    let booking_id = ObjectId::parse_str(&booking_id_str)
        .map_err(|_| AppError::bad_request("Invalid booking ID format"))?;

    let result = controllers::booking::get(&identity, &booking_id).await;

    match result {
        Ok(Some(booking)) => Ok(HttpResponse::Ok().json(util::util_serde::to_value(booking))),
        Ok(None) => Err(AppError::not_found("Booking not found")),
        Err(error) => Err(error),
    }
}

pub fn configure(config: &mut web::ServiceConfig) {
    config
        .service(create)
        .service(list)
        .service(update)
        .service(get);
}
