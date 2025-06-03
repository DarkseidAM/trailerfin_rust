use anyhow::{anyhow, Context};
use std::{path::PathBuf, sync::Arc};
use config::Config;
use tracing::info;

#[derive(Debug, Default, serde::Deserialize, PartialEq)]
pub struct AppConfig {
    pub scan_path: String,
    pub video_filename: String,
    pub should_schedule: bool,
    pub schedule: Option<String>,
    pub user_agent: String,
    pub threads: usize,
}

pub fn load_config() -> anyhow::Result<Arc<AppConfig>> {
    let config = Config::builder()
        .set_default("scan_path", "/mnt/plex")?
        .set_default("user_agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/124.0.0.0")?
        .set_default("should_schedule", false)?
        .set_default("video_filename", "video1.strm")?
        .set_default("threads", 1)?
        .add_source(
            config::Environment::with_prefix("TRAILERFIN")
        )
        .build()?;

    let config: AppConfig = config.try_deserialize()?;

    if config.threads < 1 {
        return Err(anyhow::anyhow!("TRAILERFIN_THREADS must be greater than or equal to 1"));
    }

    if config.scan_path.is_empty() {
        return Err(anyhow::anyhow!("TRAILERFIN_SCAN_PATH must be set and cannot be empty"));
    }

    if config.user_agent.trim().is_empty() {
        return Err(anyhow::anyhow!("TRAILERFIN_USER_AGENT must be set and cannot be empty"));
    }

    if config.video_filename.trim().is_empty() {
        return Err(anyhow!("TRAILERFIN_VIDEO_FILENAME must be set and cannot be empty"));
    }

    if config.should_schedule {
        match config.schedule.as_deref().map(str::trim) {
            Some("") | None => {
                return Err(anyhow!("TRAILERFIN_SCHEDULE must be set and not empty when scheduling is enabled"));
            }
            _ => {}
        }
    }

    _ = validate_scan_path(&config.scan_path)
        .context("Invalid scan path provided. Check that it exists!")?;

    info!("Loaded configuration: {:?}", config);

    Ok(Arc::new(config))
}

fn validate_scan_path(scan_path: &str) -> anyhow::Result<PathBuf> {
    let path_buf = PathBuf::from(scan_path);
    if !path_buf.exists() || !path_buf.is_dir() {
        return Err(anyhow::anyhow!(
            "Provided scan path does not exist or is not a directory: {:?}",
            path_buf
        ));
    }
    Ok(path_buf.canonicalize().context("Failed to canonicalize scan path")?)
}