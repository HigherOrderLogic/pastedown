use std::error::Error;

use crate::extractors::Json;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Serialize)]
pub struct ApiError {
    detail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cause: Option<String>,
}

impl ApiError {
    pub fn with_detail<S>(s: S) -> Self
    where
        S: AsRef<str>,
    {
        Self {
            detail: s.as_ref().to_owned(),
            cause: None,
        }
    }

    pub fn from_error<E>(e: &E) -> Self
    where
        E: Error + ?Sized,
    {
        Self {
            detail: e.to_string(),
            cause: e.source().map(|e| e.to_string()),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        Json(self).into_response()
    }
}
