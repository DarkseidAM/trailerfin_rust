use crate::config::config::load_config;
use crate::trailers::scraper::scan_and_refresh_trailers;
use tracing::info;
use tracing_subscriber::EnvFilter;

mod config;
mod trailers;
mod scheduler;

fn init_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt().with_env_filter(filter).init();
}

#[tokio::main]
async fn main() {
    init_tracing();

    let app_config = load_config().expect("Failed to load configuration");

    if app_config.should_schedule {
        info!("Starting in scheduled mode...");
        scheduler::schedule::start_scheduler(app_config)
            .await
            .expect("Failed to start scheduler");
    } else {
        info!("Scheduling disabled: Running Once...");
        scan_and_refresh_trailers(&app_config).await
            .expect("Failed to scan and refresh trailers");
    }
}
