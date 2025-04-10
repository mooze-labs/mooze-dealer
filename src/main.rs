use clap::Parser;
use log::{debug, info};
use log4rs;
use sqlx::postgres::PgPoolOptions;
use std::fs;
use std::path::Path;

mod models;
mod repositories;
pub mod services;
pub mod settings;
pub mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "config.toml")]
    config: String,
    #[arg(long, default_value = "log4rs.yaml")]
    log4rs: String,
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    init_logging(&args.log4rs).unwrap(); // should not fail

    info!("Starting Mooze dealer service.");
    debug!("Loading configuration");

    let config = settings::Settings::new(&args.config).expect("Could not load config file.");

    info!(
        "Connecting to PostgreSQL database at {}",
        &config.postgres.url
    );
    let conn = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.postgres.url)
        .await
        .expect("Could not connect to database.");

    info!("Starting services.");
    services::start_services(conn, config)
        .await
        .expect("Could not start services.");

    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for Ctrl-C");
    info!("\n[*] Shutdown signal received, terminating.");

    info!("Service shutting down");
}

fn init_logging(path: &str) -> Result<(), anyhow::Error> {
    if !Path::new("logs").exists() {
        fs::create_dir("logs")?;
    }

    match log4rs::init_file(path, Default::default()) {
        Ok(_) => {
            println!("[*] Logging initialized successfully.");
            Ok(())
        }
        Err(e) => {
            println!("[ERROR] Failed to initialize logging: {}", e);
            Err(anyhow::anyhow!("Could not initialize logging: {}", e))
        }
    }
}
