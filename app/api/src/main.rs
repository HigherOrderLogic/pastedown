#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

mod app;
mod db;
mod extractors;
mod responses;
mod routes;
mod structs;
mod utils;

use std::{env, io, net::Ipv4Addr, sync::Arc};

use crate::app::{__AppState, create_app};
use dotenvy::dotenv;
use tokio::net::TcpListener;
use tracing_subscriber::{
    fmt, layer::SubscriberExt, registry as tracing_registry, util::SubscriberInitExt, EnvFilter,
};

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    dotenv().expect("Failed to load .env file");

    tracing_registry()
        .with(fmt::layer().with_target(false))
        .with(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,axum=trace", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .try_init()
        .expect("Failed to setup logging");

    tracing::info!("Binary built on {}", env!("BUILD_TIMESTAMP"));

    let app = create_app().with_state(Arc::new(__AppState::init().await));

    let listener = TcpListener::bind((
        env::var("HOST")
            .map(|val| val.parse().expect("Invalid HOST env variable"))
            .unwrap_or(Ipv4Addr::UNSPECIFIED),
        env::var("PORT")
            .map(|val| val.parse().expect("Invalid PORT env variable"))
            .unwrap_or(10001),
    ))
    .await?;

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            wait_until_shutdown().await;
            tracing::info!("Received shutdown signal, shutting down gracefully");
        })
        .await
        .inspect_err(|e| tracing::error!("Cannot start server: {}", e))?;

    Ok(())
}

#[cfg(unix)]
async fn wait_until_shutdown() {
    use tokio::signal::unix::{signal, SignalKind};

    let mut sigterm = signal(SignalKind::terminate()).expect("Failed to install SIGTERM handler");
    let mut sigint = signal(SignalKind::interrupt()).expect("Failed to install SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => {},
        _ = sigint.recv() => {},
    }
}

#[cfg(not(unix))]
async fn wait_until_shutdown() {
    use tokio::signal::windows;

    let mut ctrl_c = windows::ctrl_c().expect("Failed to install Ctrl+C handler");
    let mut ctrl_break = windows::ctrl_break().expect("Failed to install Ctrl+Break handler");

    tokio::select! {
        _ = ctrl_c.recv() => {},
        _ = ctrl_break.recv() => {},
    }
}
