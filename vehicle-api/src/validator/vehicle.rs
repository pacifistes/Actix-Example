use validator::Validate;

use crate::authentication::identity::{Identity, Role};
use crate::error::{AppError, AppResult};
use crate::models::{Brand, CarModel, FuelType, UpdateVehicleRequest, Vehicle, VehicleMetadata};

/// Validate Tesla constraints on metadata
pub async fn validate_metadata(
    _identity: &Identity,
    metadata: &VehicleMetadata,
) -> Result<(), String> {
    match metadata {
        VehicleMetadata::Car(car_metadata) => {
            // If it's a Tesla model, fuel type must be Electric
            if matches!(
                car_metadata.model,
                CarModel::MODEL_S
                    | CarModel::MODEL_3
                    | CarModel::MODEL_X
                    | CarModel::MODEL_Y
                    | CarModel::CYBERTRUCK
                    | CarModel::ROADSTER
            ) {
                if car_metadata.fuel_type != FuelType::ELECTRIC {
                    return Err("Tesla vehicles must have Electric fuel type.".to_string());
                }
            }
        }
        VehicleMetadata::Motorbike(_) => {
            // No specific Tesla constraints for motorbikes
        }
    }
    Ok(())
}

/// Validate brand and model compatibility
pub async fn validate_brand_model(brand: &Brand, metadata: &VehicleMetadata) -> Result<(), String> {
    match (brand, metadata) {
        // Car brands with car metadata
        (Brand::TESLA, VehicleMetadata::Car(car_metadata)) => {
            if !matches!(
                car_metadata.model,
                CarModel::MODEL_S
                    | CarModel::MODEL_3
                    | CarModel::MODEL_X
                    | CarModel::MODEL_Y
                    | CarModel::CYBERTRUCK
                    | CarModel::ROADSTER
            ) {
                return Err("Invalid model for Tesla brand.".to_string());
            }
        }

        (Brand::MERCEDES, VehicleMetadata::Car(car_metadata)) => {
            if !matches!(
                car_metadata.model,
                CarModel::A_CLASS
                    | CarModel::C_CLASS
                    | CarModel::E_CLASS
                    | CarModel::S_CLASS
                    | CarModel::G_CLASS
                    | CarModel::GLC
                    | CarModel::GLE
                    | CarModel::AMG_GT
            ) {
                return Err("Invalid model for Mercedes brand.".to_string());
            }
        }

        // Motorbike brands with motorbike metadata (allow all combinations for now)
        (
            Brand::HONDA
            | Brand::YAMAHA
            | Brand::KAWASAKI
            | Brand::DUCATI
            | Brand::BMW
            | Brand::HARLEY_DAVIDSON,
            VehicleMetadata::Motorbike(_),
        ) => {
            // All motorbike brands are valid with motorbike metadata
        }

        // Invalid combinations: car brands with motorbike metadata
        (Brand::TESLA | Brand::MERCEDES, VehicleMetadata::Motorbike(_)) => {
            return Err(format!(
                "{:?} is a car brand and cannot be used with motorbike metadata.",
                brand
            ));
        }

        // Invalid combinations: motorbike brands with car metadata
        (
            Brand::HONDA
            | Brand::YAMAHA
            | Brand::KAWASAKI
            | Brand::DUCATI
            | Brand::BMW
            | Brand::HARLEY_DAVIDSON,
            VehicleMetadata::Car(_),
        ) => {
            return Err(format!(
                "{:?} is a motorbike brand and cannot be used with car metadata.",
                brand
            ));
        }
    }
    Ok(())
}

pub(crate) fn check_vehicle_type_permission(
    identity: &Identity,
    vehicle: &Vehicle,
) -> Result<(), AppError> {
    match identity.role {
        Role::Admin => Ok(()), // Admin can manage all vehicle types
        Role::CarManager => {
            if matches!(vehicle.metadata, VehicleMetadata::Car(_)) {
                Ok(())
            } else {
                Err(AppError::forbidden("CarManager can only manage cars."))
            }
        }
        Role::MotorbikeManager => {
            if matches!(vehicle.metadata, VehicleMetadata::Motorbike(_)) {
                Ok(())
            } else {
                Err(AppError::forbidden(
                    "MotorbikeManager can only manage motorbikes.",
                ))
            }
        }
        _ => Err(AppError::forbidden(
            "Insufficient permissions to manage vehicles.",
        )),
    }
}

pub(crate) fn validate_update_vehicle(
    identity: &Identity,
    vehicle: &Vehicle,
    request: &UpdateVehicleRequest,
) -> AppResult<()> {
    request
        .validate()
        .map_err(|e| AppError::bad_request(e.to_string()))?;

    check_vehicle_type_permission(identity, vehicle)
}
