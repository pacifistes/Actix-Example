use std::{ops::Deref, pin::Pin};

use actix_web::{web, FromRequest, HttpMessage};
use futures::Future;
use serde::de::DeserializeOwned;
use validator::Validate;

use crate::authentication::identity::Identity;

use super::CustomValidateTrait;

#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T> Json<T> {
    /// Deconstruct to an inner value
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl<T> AsRef<T> for Json<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Json<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> FromRequest for Json<T>
where
    T: DeserializeOwned + Validate + CustomValidateTrait + 'static,
{
    type Error = Box<dyn std::error::Error>;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    #[allow(clippy::type_complexity)]
    fn from_request(
        req: &actix_web::HttpRequest,
        payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let identity = req.extensions().get::<Identity>().unwrap().clone();
        let json = web::Json::<T>::from_request(req, payload);
        Box::pin(async move {
            let json: T = json.await?.into_inner();
            Validate::validate(&json)?;
            CustomValidateTrait::validate(&json, &identity).await?;
            Ok(Json(json))
        })
    }
}
