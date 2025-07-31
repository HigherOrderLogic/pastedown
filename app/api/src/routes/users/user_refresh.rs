use axum::http::StatusCode;

use crate::{
    extractors::Jwt,
    responses::{ApiError, ApiResult, WithStatusCode},
    structs::JwtClaims,
};

pub async fn handler(Jwt(jwt): Jwt<JwtClaims>) -> ApiResult<()> {
    jwt.validate_token(true)
        .map_err(|e| ApiError::from_error(&e).with_status_code(StatusCode::UNAUTHORIZED))?;

    todo!()
}
