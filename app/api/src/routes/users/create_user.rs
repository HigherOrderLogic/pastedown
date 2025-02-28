use crate::{
    app::AppState,
    db::Users,
    extractor::{Json, Validated},
    response::{ApiError, ApiResult, WithStatusCode},
    utils::get_argon2_ctx,
};
use argon2::{password_hash::SaltString, PasswordHasher};
use axum::{extract::State, http::StatusCode};
use rand::rngs::OsRng;
use sea_query::{Expr, OnConflict, PostgresQueryBuilder, Query};
use sea_query_postgres::PostgresBinder;
use serde::Deserialize;
use tokio::task::spawn_blocking;
use validator::Validate;

fn hash_password(password: String) -> String {
    let arg2 = get_argon2_ctx();
    let salt = SaltString::generate(&mut OsRng);

    arg2.hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

#[derive(Deserialize, Validate)]
pub struct Payload {
    #[validate(length(min = 2, message = "Value must contain at least 2 characters"))]
    pub username: String,
    #[validate(email(message = "Value must be a valid email"))]
    pub email: Option<String>,
    #[validate(length(min = 8, message = "Value must contain at least 8 characters"))]
    pub password: String,
}

pub async fn handler(
    State(state): State<AppState>,
    Validated(Json(payload)): Validated<Json<Payload>>,
) -> ApiResult<()> {
    let conn = state.db.get().await.unwrap();

    let hashed_password = hash_password(payload.password);
    let jwt_id = spawn_blocking(move || state.snowflake.next_id().unwrap())
        .await
        .unwrap()
        .to_string();

    let (statement, params) = Query::insert()
        .into_table(Users::table_name())
        .columns([Users::Username, Users::Password, Users::Email, Users::JwtId])
        .values_panic([
            payload.username.into(),
            hashed_password.into(),
            payload.email.clone().into(),
            jwt_id.into(),
        ])
        .on_conflict(
            OnConflict::new()
                .exprs([Expr::col(Users::Username), Expr::col(Users::Email)])
                .do_nothing()
                .to_owned(),
        )
        .build_postgres(PostgresQueryBuilder);

    let rows_modified = conn.execute(&statement, &params.as_params()).await.unwrap();

    if rows_modified < 1 {
        Err(ApiError::with_detail(if payload.email.is_none() {
            "Conflicted username"
        } else {
            "Conflicted username or email"
        })
        .with_status_code(StatusCode::CONFLICT))
    } else {
        Ok(().with_status_code(StatusCode::CREATED))
    }
}
