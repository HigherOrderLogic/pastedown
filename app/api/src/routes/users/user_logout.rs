use crate::{
    app::AppState,
    db::Users,
    extractors::Jwt,
    responses::{ApiError, ApiResult, WithStatusCode},
    structs::JwtClaims,
};
use axum::{extract::State, http::StatusCode};
use sea_query::{Cond, Expr, PostgresQueryBuilder, Query};
use sea_query_postgres::PostgresBinder;
use tokio::task::spawn_blocking;
use uuid::Uuid;

pub async fn handler(State(state): State<AppState>, Jwt(jwt): Jwt<JwtClaims>) -> ApiResult<()> {
    jwt.validate_token(false)
        .map_err(|e| ApiError::from_error(&e).with_status_code(StatusCode::UNAUTHORIZED))?;

    let mut conn = state.db.get().await.unwrap();
    let trans = conn.transaction().await.unwrap();

    let new_jwt_id = spawn_blocking(move || state.snowflake.next_id().unwrap())
        .await
        .unwrap()
        .to_string();

    let (statement, params) = Query::update()
        .table(Users::table_name())
        .values([(Users::JwtId, new_jwt_id.into())])
        .cond_where(
            Cond::all()
                .add(Expr::col(Users::Uuid).eq(jwt.user_uuid))
                .add(Expr::col(Users::JwtId).eq(jwt.jwt_id)),
        )
        .returning(Query::returning().columns([Users::Uuid]))
        .build_postgres(PostgresQueryBuilder);

    let row = trans
        .query_opt(&statement, &params.as_params())
        .await
        .unwrap()
        .ok_or(
            ApiError::with_detail("Invalid JWT token").with_status_code(StatusCode::UNAUTHORIZED),
        )?;

    if row.get::<_, Uuid>("uuid") == jwt.user_uuid {
        trans.commit().await.unwrap();

        Err(ApiError::with_detail("Invalid JWT token").with_status_code(StatusCode::UNAUTHORIZED))
    } else {
        trans.rollback().await.unwrap();

        Ok(().with_status_code(StatusCode::OK))
    }
}
