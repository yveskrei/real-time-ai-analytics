use anyhow::{Result, Context};

// Custom modules
use client::utils::config::AppConfig;
use client::services;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> Result<()> {
    // Iniaitlize config
    let app_config = AppConfig::new()
        .context("Error loading config")?;

    services::init_services(&app_config, tokio::runtime::Handle::current())
        .await
        .context("Error initiating services")?;

    Ok(())
}