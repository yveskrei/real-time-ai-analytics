use axum::{Router};
use std::net::SocketAddr;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use anyhow::{Result, Context};

// Custom modules
use client::utils::config::AppConfig;
use client::statistics;
use client::handlers;
use client::inference;

#[tokio::main]
async fn main() -> Result<()> {
    // Iniaitlize config
    let app_config = AppConfig::new()
        .context("Error loading config")?;

    // Initiate inference client
    inference::init_inference_models(&app_config)
        .await
        .context("Error initiating inference model")?;

    inference::start_models_instances(&app_config)
        .await
        .context("Error initiating inference model instances")?;

    // Initiate statistics
    statistics::init_statistics()
        .context("Error initiating statistics")?;

    // Build API application
    let app = Router::new()
        .merge(handlers::routes())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http());

    // Register port for application
    let addr = SocketAddr::from((
        [127, 0, 0, 1], 
        app_config.port()
    ));
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .context("Unable to register dedicated port")?;

    tracing::info!("Server running on http://{}", addr);

    // Start application
    axum::serve(listener, app)
        .await
        .context("Cannot start application")?;

    Ok(())
}