use std::{any::Any, env, sync::Arc, time::Duration};

use crate::{
    response::{ApiError, WithStatusCode},
    routes,
};
use axum::{
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Router,
};
use chrono::DateTime;
use deadpool_postgres::{tokio_postgres::NoTls, Config as PoolConfig, Pool as DbPool, Runtime};
use jsonwebtoken::Algorithm;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use snowflake_me::Snowflake;
use tower::ServiceBuilder;
use tower_http::{
    catch_panic::CatchPanicLayer,
    compression::CompressionLayer,
    cors::{AllowHeaders, CorsLayer},
    timeout::TimeoutLayer,
    trace::{DefaultOnFailure, TraceLayer},
};
use tracing::Level;

pub struct JwtConfig {
    pub secret: String,
    pub expires_in: Duration,
    pub algorithm: Algorithm,
}

impl JwtConfig {
    pub fn init() -> Self {
        Self {
            secret: env::var("JWT_SECRET").unwrap_or(
                thread_rng()
                    .sample_iter(Alphanumeric)
                    .take(32)
                    .map(char::from)
                    .collect(),
            ),
            expires_in: Duration::from_secs(
                env::var("JWT_EXPIRES_IN")
                    .map(|v| v.parse().expect("Invalid JWT_EXPIRES_IN env variable"))
                    .unwrap_or(15 * 60),
            ),
            algorithm: Algorithm::HS256,
        }
    }
}

pub struct __AppState {
    pub jwt_config: Arc<JwtConfig>,
    pub db: DbPool,
    pub snowflake: Snowflake,
}

pub type AppState = Arc<__AppState>;

impl __AppState {
    pub async fn init() -> Self {
        let db_pool = PoolConfig::new()
            .create_pool(Some(Runtime::Tokio1), NoTls)
            .expect("Failed to create database pool");

        Self {
            jwt_config: Arc::new(JwtConfig::init()),
            db: db_pool,
            snowflake: Snowflake::builder()
                .start_time(
                    DateTime::parse_from_str("01 Jan 2020", "%d %b %Y")
                        .unwrap()
                        .into(),
                )
                .machine_id(&|| Ok(0))
                .machine_id(&|| Ok(0))
                .finalize()
                .unwrap(),
        }
    }
}

pub fn create_app() -> Router<AppState> {
    Router::new()
        .layer(
            ServiceBuilder::new()
                .layer(TimeoutLayer::new(Duration::from_secs(10)))
                .layer(CatchPanicLayer::custom(handle_panic))
                .layer(
                    TraceLayer::new_for_http()
                        .on_failure(DefaultOnFailure::new().level(Level::WARN)),
                )
                .layer(
                    CorsLayer::new()
                        .allow_methods([
                            Method::GET,
                            Method::POST,
                            Method::PUT,
                            Method::PATCH,
                            Method::DELETE,
                        ])
                        .allow_credentials(true)
                        .allow_headers(AllowHeaders::any()),
                )
                .layer(
                    CompressionLayer::new()
                        .br(true)
                        .gzip(true)
                        .no_deflate()
                        .no_zstd(),
                ),
        )
        .route("/ping", get(|| async { StatusCode::OK }))
        .nest("/pastes", routes::pastes::router())
        .nest("/users", routes::users::router())
}

fn handle_panic(err: Box<dyn Any + Send + 'static>) -> Response {
    let detail = if let Some(s) = err.downcast_ref::<String>() {
        s.as_str()
    } else if let Some(s) = err.downcast_ref::<&str>() {
        s
    } else {
        "No error details"
    };

    tracing::error!("Handler panic: {}", detail);

    ApiError::with_detail(detail)
        .with_status_code(StatusCode::INTERNAL_SERVER_ERROR)
        .into_response()
}
