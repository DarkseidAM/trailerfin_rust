use crate::scrapers::traits::TrailerScraper;
use std::path::{PathBuf};

use anyhow::Result;
use regex::Regex;
use std::sync::Arc;
use async_trait::async_trait;

use tracing::{error, info, warn};
use crate::caching::tmdb_to_imdb_cache::TmdbToImdbCache;
use crate::configuration::configuration_provider::AppConfig;
use crate::request_clients::get_tmdb_client;
use crate::request_clients::tmdb_client::external_ids_endpoints;
use crate::scrapers::media_directories::{process_media_folders, FolderType, BACKDROPS_FOLDER};
use crate::scrapers::imdb_trailers::ImdbTrailerScraper;

#[derive(Debug)]
pub struct TmdbTrailerScraper {
    pub imdb_trailer_scraper: Arc<ImdbTrailerScraper>,
    pub tmdb_to_imdb_cache: Arc<TmdbToImdbCache>,
}

// This will be replaced with configurable regex

#[async_trait]
impl TrailerScraper for TmdbTrailerScraper {
    async fn scan_and_refresh_trailers(self: Arc<Self>, config: &Arc<AppConfig>) -> Result<()> {
        self.perform_scan_and_refresh_tmdb(config).await
    }

    async fn process_path(&self, path: PathBuf, config: Arc<AppConfig>, folder_type: FolderType) {
        self.process_path_internal(path, config, folder_type).await;
    }
}

impl TmdbTrailerScraper {
    pub async fn perform_scan_and_refresh_tmdb(self: Arc<Self>, app_config: &Arc<AppConfig>) -> Result<()> {
        process_media_folders(app_config, self as Arc<dyn TrailerScraper>).await
    }

    async fn process_path_internal(
        &self,
        path: PathBuf,
        config: Arc<AppConfig>,
        folder_type: FolderType,
    ) {
        if let Some(path_str) = path.to_str() {
            // Use configurable regex pattern
            let regex = match Regex::new(&config.tmdb_id_regex) {
                Ok(regex) => regex,
                Err(e) => {
                    error!("Invalid TMDB ID regex pattern: {}", e);
                    return;
                }
            };
            
            if let Some(cap) = regex.captures(path_str) {
                let tmdb_id = &cap[1];
                let backdrops_path = path.join(BACKDROPS_FOLDER);
                let strm_path = backdrops_path.join(&config.video_filename);

                if let Ok(expired) = self.imdb_trailer_scraper.is_strm_expired(&strm_path) {
                    if !expired {
                        info!("Trailer still valid for {} in {:?}", tmdb_id, path);
                        return;
                    }
                }

                info!("Refreshing trailer for {} in {:?}", tmdb_id, path);

                let imdb_id = match self.get_imdb_id(tmdb_id, folder_type).await {
                    Ok(id) => id,
                    Err(e) => {
                        warn!("Failed to retrieve IMDb ID for {}: {:?}", tmdb_id, e);
                        return; // just early-exit the method
                    }
                };

                self.imdb_trailer_scraper
                    .refresh_imdb_trailer(&imdb_id, path.clone(), config)
                    .await;
            } else {
                warn!("No TMDB ID found in path: {:?}", path);
            }
        }
    }

    async fn get_imdb_id(&self, tmdb_id: &str, folder_type: FolderType) -> Result<String> {
        if let Some(imdb_id) = self.tmdb_to_imdb_cache.try_get_imdb_id(tmdb_id)? {
            return Ok(imdb_id);
        }

        info!("No IMDB ID found in local cache for TMDB ID: {}", tmdb_id);
        let client = get_tmdb_client();

        let external_ids = match folder_type {
            FolderType::TvShow => {
                info!("Fetching IMDB ID for TV Show TMDB ID: {}", tmdb_id);
                external_ids_endpoints::ExternalIds::Tv(client.external_ids().get_for_tv(tmdb_id).await?)
            }
            FolderType::Movie => {
                info!("Fetching IMDB ID for Movie TMDB ID: {}", tmdb_id);
                external_ids_endpoints::ExternalIds::Movie(client.external_ids().get_for_movie(tmdb_id).await?)
            }
        };

        if let Some(imdb_id) = external_ids.imdb_id() {
            self.tmdb_to_imdb_cache.add(tmdb_id, imdb_id)?;
            Ok(imdb_id.to_string())
        } else {
            warn!("No IMDB ID found for TMDB ID: {}", tmdb_id);
            Err(anyhow::anyhow!("No IMDB ID found for TMDB ID: {}", tmdb_id))
        }
    }
}