use anyhow::{anyhow, Context};
use std::{path::PathBuf, sync::Arc};
use std::path::Path;
use config::Config;
use tracing::{info};
use serde::de::{self, Deserializer};
use serde::Deserialize;

const DATASOURCES: [&str; 2] = ["IMDB", "TMDB"];

fn case_insensitive_datasource<'de, D>(deserializer: D) -> Result<DataSource, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.to_lowercase().as_str() {
        "imdb" => Ok(DataSource::Imdb),
        "tmdb" => Ok(DataSource::Tmdb),
        other => Err(de::Error::custom(format!("invalid TRAILERFIN_DATA_SOURCE: {}. Must be one of: {:?}", other, DATASOURCES)))
    }
}

fn validate_path(path: &str, name: &str) -> anyhow::Result<PathBuf> {
    let path_buf = PathBuf::from(path);
    if !path_buf.exists() || !path_buf.is_dir() {
        return Err(anyhow::anyhow!(
                "Provided path for {} does not exist or is not a directory: {:?}",
                name,
                path_buf
            ));
    }
    Ok(path_buf.canonicalize().with_context(|| format!("Failed to canonicalize path for {}: {:?}", name, path_buf))?)
}

fn deserialize_trimmed_csv<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(s.split(',')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect())
}

#[derive(Debug, Default, serde::Deserialize, PartialEq)]
pub enum DataSource {
    #[default]
    Imdb,
    Tmdb,
}

#[derive(Debug, Default, serde::Deserialize, PartialEq)]
pub struct AppConfig {
    pub scan_path: String,
    pub video_filename: String,
    pub should_schedule: bool,
    pub schedule: Option<String>,
    pub user_agent: String,
    pub threads: usize,
    pub cache_path: String,

    #[serde(default)]
    #[serde(deserialize_with = "case_insensitive_datasource")]
    pub data_source: DataSource,

    pub imdb_rate_limit: String,
    pub tmdb_rate_limit: String,
    pub tmdb_api_key: Option<String>,

    #[serde(default, deserialize_with = "deserialize_trimmed_csv")]
    pub tv_folders: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_trimmed_csv")]
    pub movie_folders: Vec<String>,
}

#[derive(Debug)]
pub struct ConfigurationProvider;

impl ConfigurationProvider {
    pub fn load_config() -> anyhow::Result<Arc<AppConfig>> {
        let config = Config::builder()
            .set_default("scan_path", "/mnt/plex")?
            .set_default("user_agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 Chrome/124.0.0.0")?
            .set_default("should_schedule", false)?
            .set_default("video_filename", "video1.strm")?
            .set_default("threads", 1)?
            .set_default("cache_path", "/config")?
            .set_default("data_source", "IMDB")?
            .set_default("imdb_rate_limit", "30/minute")?
            .set_default("tmdb_rate_limit", "50/second")?
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

        if config.data_source == DataSource::Tmdb {
            match config.tmdb_api_key.as_deref().map(str::trim) {
                Some("") | None => {
                    return Err(anyhow!("TRAILERFIN_TMDB_API_KEY must be set and not empty when datasource is set to TMDB"));
                }
                _ => {}
            }
        }

        if config.cache_path.trim().is_empty() {
            return Err(anyhow!("TRAILERFIN_CACHE_PATH must be set and cannot be empty"));
        }

        _ = validate_path(&config.scan_path, "TRAILERFIN_SCAN_PATH")?;
        _ = validate_path(&config.cache_path, "TRAILERFIN_CACHE_PATH")?;

        if config.tv_folders.is_empty() && config.movie_folders.is_empty() {
            return Err(anyhow!("At least one of TRAILERFIN_TV_FOLDERS or TRAILERFIN_MOVIE_FOLDERS must be set and non-empty"));
        }

        for folder in config.tv_folders.iter().chain(config.movie_folders.iter()) {
            let full_path = Path::new(&config.scan_path).join(folder);
            validate_path(full_path.to_str().unwrap(), &format!("subfolder: {}", folder))?;
        }

        info!("Loaded configuration: {:?}", config);

        Ok(Arc::new(config))
    }
}