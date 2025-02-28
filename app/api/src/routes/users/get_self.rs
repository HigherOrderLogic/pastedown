use crate::{
    app::AppState,
    extractor::Jwt,
    response::{ApiError, ApiResult, WithStatusCode},
    structs::JwtClaims,
};
use axum::{extract::State, http::StatusCode};

pub async fn handler(State(state): State<AppState>, Jwt(jwt): Jwt<JwtClaims>) -> ApiResult<()> {
    jwt.validate_token(false)
        .map_err(|e| ApiError::from_error(&e).with_status_code(StatusCode::UNAUTHORIZED))?;

    let conn = state.db.get().await.unwrap();

    conn.execute("", &[]).await.unwrap();

    todo!()
}
