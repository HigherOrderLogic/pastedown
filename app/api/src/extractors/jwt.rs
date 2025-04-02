use crate::{
    app::AppState,
    responses::{ApiError, WithStatusCode},
};
use axum::{
    extract::FromRequestParts,
    http::{header, request::Parts, StatusCode},
    response::{IntoResponse, Response},
};
use jsonwebtoken::{decode, errors, DecodingKey, Validation};
use serde::de::DeserializeOwned;

pub enum JwtRejection {
    NoAuthorizationHeader,
    DecodeError(errors::Error),
}

impl IntoResponse for JwtRejection {
    fn into_response(self) -> Response {
        match self {
            Self::NoAuthorizationHeader => ApiError::with_detail("No Authorization header found"),
            Self::DecodeError(e) => ApiError::from_error(&e),
        }
        .with_status_code(StatusCode::UNAUTHORIZED)
        .into_response()
    }
}

pub struct Jwt<T>(pub T);

impl<T> FromRequestParts<AppState> for Jwt<T>
where
    T: DeserializeOwned,
{
    type Rejection = JwtRejection;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(JwtRejection::NoAuthorizationHeader)?;

        let decoded = decode::<T>(
            token,
            &DecodingKey::from_secret(state.jwt_config.secret.as_ref()),
            &Validation::new(state.jwt_config.algorithm),
        )
        .map_err(JwtRejection::DecodeError)?;

        Ok(Self(decoded.claims))
    }
}
