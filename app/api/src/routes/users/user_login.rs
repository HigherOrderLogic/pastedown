use crate::{
    app::AppState,
    db::Users,
    extractor::{Json, Validated},
    response::{ApiError, ApiResult, WithStatusCode},
    structs::{JwtClaims, Token},
    utils::get_argon2_ctx,
};
use argon2::{PasswordHash, PasswordVerifier};
use axum::{extract::State, http::StatusCode};
use chrono::Utc;
use jsonwebtoken::{encode, EncodingKey, Header};
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_postgres::PostgresBinder;
use serde::Deserialize;
use validator::Validate;

fn verify_password(password: String, hash: &str) -> bool {
    let arg2 = get_argon2_ctx();

    PasswordHash::new(hash).is_ok_and(|h| arg2.verify_password(password.as_bytes(), &h).is_ok())
}

#[derive(Deserialize, Validate)]
pub struct AuthUserPayload {
    #[validate(length(min = 2, message = "Value must contain at least 2 characters"))]
    pub username: String,
    #[validate(length(min = 8, message = "Value must contain at least 8 characters"))]
    pub password: String,
}

pub async fn handler(
    State(state): State<AppState>,
    Validated(Json(payload)): Validated<Json<AuthUserPayload>>,
) -> ApiResult<Json<Token>> {
    let conn = state.db.get().await.unwrap();

    let (statement, params) = Query::select()
        .from(Users::table_name())
        .columns([Users::Uuid, Users::Username, Users::Password, Users::JwtId])
        .cond_where(Expr::col(Users::Username).eq(payload.username))
        .build_postgres(PostgresQueryBuilder);

    let row = conn
        .query_opt(&statement, &params.as_params())
        .await
        .unwrap()
        .ok_or(
            ApiError::with_detail("Invalid username provided")
                .with_status_code(StatusCode::UNAUTHORIZED),
        )?;

    if !verify_password(payload.password, row.get("password")) {
        return Err(ApiError::with_detail("Invalid password provided")
            .with_status_code(StatusCode::UNAUTHORIZED));
    }

    let user_uuid = row.get("uuid");
    let jwt_id: String = row.get("jwt_id");
    let issues_at = Utc::now();
    let expires_at = issues_at + state.jwt_config.expires_in;

    Ok(Json(Token {
        access_token: encode(
            &Header::new(state.jwt_config.algorithm),
            &JwtClaims {
                user_uuid,
                jwt_id: jwt_id.clone(),
                issues_at,
                expires_at: Some(expires_at),
                is_refresh: false,
            },
            &EncodingKey::from_secret(state.jwt_config.secret.as_ref()),
        )
        .unwrap(),
        refresh_token: encode(
            &Header::new(state.jwt_config.algorithm),
            &JwtClaims {
                user_uuid,
                jwt_id,
                issues_at,
                expires_at: None,
                is_refresh: true,
            },
            &EncodingKey::from_secret(state.jwt_config.secret.as_ref()),
        )
        .unwrap(),
    })
    .with_status_code(StatusCode::OK))
}
