use anyhow::Result;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};

use crate::config::config::AppConfig;
use crate::trailers::scraper::scan_and_refresh_trailers;

pub type ScanFn = Arc<dyn Fn(Arc<AppConfig>) -> ScanFuture + Send + Sync>;

pub type ScanFuture = std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>;

pub async fn setup_scheduler(
    app_config: Arc<AppConfig>,
    scan_fn: Option<ScanFn>,
) -> Result<JobScheduler> {
    let sched = JobScheduler::new().await?;
    let schedule_expr = app_config.schedule.clone().unwrap_or_else(|| "0 0 * * *".to_string());

    let scan_handler: ScanFn = scan_fn.unwrap_or_else(|| {
        Arc::new(|config: Arc<AppConfig>| {
            Box::pin(async move {
                scan_and_refresh_trailers(&config)
                    .await
                    .map_err(|e| anyhow::anyhow!(e))
            })
        })
    });

    let config = Arc::clone(&app_config);
    let scan_handler_clone = Arc::clone(&scan_handler);

    let job = Job::new_async(&schedule_expr, move |_uuid, _l| {
        let config = Arc::clone(&config);
        let scan_handler = Arc::clone(&scan_handler_clone);
        Box::pin(async move {
            info!("Running scheduled trailer scan...");
            if let Err(err) = scan_handler(config).await {
                error!("Scheduled scan failed: {err}");
            }
        })
    })?;

    sched.add(job).await?;
    Ok(sched)
}

pub async fn start_scheduler(app_config: Arc<AppConfig>) -> Result<()> {
    let sched = setup_scheduler(app_config.clone(), None).await?;
    info!("Scheduler started with schedule: {}", app_config.schedule.as_deref().unwrap_or("No schedule"));

    // Optionally run once on startup
    let config = Arc::clone(&app_config);
    tokio::spawn(async move {
        if let Err(err) = scan_and_refresh_trailers(&config).await {
            error!("Initial trailer scan failed: {err}");
        }
    });

    sched.start().await?;
    tokio::signal::ctrl_c().await?;
    Ok(())
}