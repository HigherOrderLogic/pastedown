use axum::{extract::State, http::StatusCode};
use sea_query::{Cond, Expr, PostgresQueryBuilder, Query};
use sea_query_postgres::PostgresBinder;

use crate::{
    app::AppState,
    db::Users,
    extractors::{Json, Jwt},
    responses::{ApiError, ApiResult, WithStatusCode},
    structs::{JwtClaims, User},
};

pub async fn handler(
    State(state): State<AppState>,
    Jwt(jwt): Jwt<JwtClaims>,
) -> ApiResult<Json<User>> {
    jwt.validate_token(false)
        .map_err(|e| ApiError::from_error(&e).with_status_code(StatusCode::UNAUTHORIZED))?;

    let conn = state.db.get().await.unwrap();

    let (statement, params) = Query::select()
        .from(Users::table_name())
        .columns([Users::Uuid, Users::Username, Users::Email, Users::IsAdmin])
        .cond_where(
            Cond::all()
                .add(Expr::col(Users::Uuid).eq(jwt.user_uuid))
                .add(Expr::col(Users::JwtId).eq(jwt.jwt_id)),
        )
        .limit(1)
        .build_postgres(PostgresQueryBuilder);

    let row = conn
        .query_opt(&statement, &params.as_params())
        .await
        .unwrap()
        .ok_or(ApiError::with_detail("Unknown user").with_status_code(StatusCode::NOT_FOUND))?;

    Ok(Json(User {
        uuid: row.get(Users::Uuid.column_name().as_str()),
        username: row.get(Users::Username.column_name().as_str()),
        email: row.get(Users::Email.column_name().as_str()),
        is_admin: row.get(Users::IsAdmin.column_name().as_str()),
    })
    .with_status_code(StatusCode::OK))
}
