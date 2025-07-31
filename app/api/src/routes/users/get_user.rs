use axum::{
    extract::{Path, State},
    http::StatusCode,
};
use sea_query::{Expr, PostgresQueryBuilder, Query};
use sea_query_postgres::PostgresBinder;
use uuid::Uuid;

use crate::{
    app::AppState,
    db::Users,
    extractors::Json,
    responses::{ApiError, ApiResult, WithStatusCode},
    structs::User,
};

pub async fn handler(
    State(state): State<AppState>,
    Path(user_uuid): Path<Uuid>,
) -> ApiResult<Json<User>> {
    let conn = state.db.get().await.unwrap();

    let (statement, params) = Query::select()
        .from(Users::table_name())
        .columns([Users::Uuid, Users::Username])
        .cond_where(Expr::col(Users::Uuid).eq(user_uuid))
        .limit(1)
        .build_postgres(PostgresQueryBuilder);

    let row = conn
        .query_opt(&statement, &params.as_params())
        .await
        .unwrap()
        .ok_or(
            ApiError::with_detail("No user with specified uuid found")
                .with_status_code(StatusCode::NOT_FOUND),
        )?;

    Ok(Json(User {
        uuid: row.get(Users::Uuid.column_name().as_str()),
        username: row.get(Users::Username.column_name().as_str()),
        email: row.get(Users::Email.column_name().as_str()),
        is_admin: row.get(Users::IsAdmin.column_name().as_str()),
    })
    .with_status_code(StatusCode::OK))
}
