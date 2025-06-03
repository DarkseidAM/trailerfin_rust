use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use tracing::{error, info, warn};
use url::Url;
use walkdir::WalkDir;
use lazy_static::lazy_static;
use std::fs::File;
use std::io::{Read, Write};
use std::string::ToString;
use std::sync::Arc;
use futures::stream::FuturesUnordered;
use futures::StreamExt;
use tokio::sync::Semaphore;
use crate::config::config::AppConfig;

const BACKDROPS_FOLDER: &str = "backdrops";
const VIDEO_PROPS_PATH: &str = "/props/pageProps/videoPlaybackData/video/playbackURLs";
const SCRIPT_SELECTOR: &str = "script#\\__NEXT_DATA__";
const VIDEO_SELECTOR: &str = "a[href*=\"/video/vi\"]";
const VIDEO_MIME_TYPE: &str = "videoMimeType";
const IMDB_URL: &str = "https://www.imdb.com";
const HREF_ATTR: &str = "href";
const VIDEO_DEFINITION_ATTR: &str = "videoDefinition";
const TRAILER: &str = "trailer";
const URL: &str = "url";
const TYPE_QUERY: &str = "#t=8";

lazy_static! {
    static ref IMDB_ID_REGEX: Regex =
        Regex::new(r"\{imdb-(tt\d+)\}")
            .expect("Failed to compile IMDB ID Regex");
}

pub async fn scan_and_refresh_trailers(app_config: &Arc<AppConfig>) -> Result<()> {
    let scan_path = PathBuf::from(&app_config.scan_path).canonicalize()?;
    if !scan_path.exists() {
        error!("Provided path does not exist: {:?}", scan_path);
        return Ok(());
    }

    let files: Vec<_> = WalkDir::new(&scan_path)
        .min_depth(1)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_type().is_dir()
                && e.path().file_name().map_or(true, |name| name != BACKDROPS_FOLDER)
        })
        .collect();

    if files.len() == 0 {
        warn!("No directories found in the scan path: {:?}", scan_path);
        return Ok(());
    }

    let client = Arc::new(Client::builder()
        .user_agent(&app_config.user_agent)
        .build()?);

    let semaphore = Arc::new(Semaphore::new(app_config.threads));
    let mut tasks = FuturesUnordered::new();

    for entry in files {
        let permit = semaphore.clone().acquire_owned().await?;
        let path = entry.path().to_path_buf();
        let client = Arc::clone(&client);
        let config = Arc::clone(&app_config);

        tasks.push(tokio::spawn(async move {
            let _permit = permit;
            if let Some(path_str) = path.to_str() {
                if let Some(cap) = (&*IMDB_ID_REGEX).captures(path_str) {
                    let imdb_id = &cap[1];
                    let backdrops_path = path.join(BACKDROPS_FOLDER);
                    let strm_path = backdrops_path.join(&config.video_filename);

                    if let Ok(expired) = is_strm_expired(&strm_path) {
                        if !expired {
                            info!("Trailer still valid for {imdb_id} in {:?}", path);
                            return;
                        }
                    }

                    info!("Refreshing trailer for {imdb_id} in {:?}", path);

                    if let Ok(Some(video_page_url)) = get_trailer_video_page_url(&client, imdb_id).await {
                        if let Ok(Some(direct_url)) = get_direct_video_url_from_page(&client, &video_page_url).await {
                            if let Err(e) = create_or_update_strm_file(&path, &config, &direct_url) {
                                error!("Failed to write .strm file: {:?}", e);
                            }
                        }
                    }
                } else {
                    warn!("No IMDB ID found in path: {:?}", path);
                }
            }
        }));
    }

    while let Some(res) = tasks.next().await {
        if let Err(e) = res {
            error!("A task panicked or failed: {:?}", e);
        }
    }

    Ok(())
}

pub fn is_strm_expired(strm_path: &Path) -> Result<bool> {
    if !strm_path.exists() {
        return Ok(true);
    }

    let mut file = File::open(strm_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    let url = Url::parse(contents.trim())?;
    if let Some(expires) = url.query_pairs().find_map(|(k, v)| {
        if k == "Expires" {
            v.parse::<i64>().ok()
        } else {
            None
        }
    }) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        Ok(now >= expires)
    } else {
        Ok(true)
    }
}

pub fn create_or_update_strm_file(folder: &Path, app_config: &AppConfig, video_url: &str) -> Result<()> {
    let backdrops = folder.join(BACKDROPS_FOLDER);
    fs::create_dir_all(&backdrops)?;
    let strm_path = backdrops.join(&app_config.video_filename);
    let mut f = File::create(&strm_path)?;
    f.write_all(video_url.as_bytes())?;
    info!("Updated {:?}", strm_path);
    Ok(())
}

async fn get_trailer_video_page_url(client: &Client, imdb_id: &str) -> Result<Option<String>> {
    let url = format!("https://www.imdb.com/title/{}/videogallery/?sort=date,asc", imdb_id);
    let res = client.get(&url).send().await?;
    if !res.status().is_success() {
        error!("Failed to fetch trailers for {} (status {})", imdb_id, res.status());
        return Ok(None);
    }

    let body = res.text().await?;
    let doc = Html::parse_document(&body);
    let selector = Result::unwrap(Selector::parse(VIDEO_SELECTOR));

    for el in doc.select(&selector) {
        let text = el.text().collect::<String>().to_lowercase();
        if text.contains(TRAILER) {
            if let Some(href) = el.value().attr(HREF_ATTR) {
                return Ok(Some(format!("{}{}",IMDB_URL, href)));
            }
        }
    }

    if let Some(el) = doc.select(&selector).next() {
        if let Some(href) = el.value().attr(HREF_ATTR) {
            return Ok(Some(format!("{}{}",IMDB_URL, href)));
        }
    }

    warn!("No video found for {}", imdb_id);
    Ok(None)
}

async fn get_direct_video_url_from_page(client: &Client, video_page_url: &str) -> Result<Option<String>> {
    let res = client.get(video_page_url).send().await?;
    if !res.status().is_success() {
        error!("Failed to fetch video page: {} (status {})", video_page_url, res.status());
        return Ok(None);
    }

    let body = res.text().await?;
    let doc = Html::parse_document(&body);

    let script_selector = Selector::parse(SCRIPT_SELECTOR).unwrap();
    let script_tag = doc.select(&script_selector).next();

    if let Some(tag) = script_tag {
        if let Some(json_text) = tag.inner_html().lines().next() {
            let data: serde_json::Value = serde_json::from_str(json_text.trim())?;
            if let Some(playbacks) = data.pointer(VIDEO_PROPS_PATH) {
                let mut mp4_urls = vec![];

                for entry in playbacks.as_array().unwrap() {
                    if entry.get(VIDEO_MIME_TYPE) == Some(&serde_json::Value::String("MP4".to_string())) {
                        mp4_urls.push(entry);
                    }
                }

                if !mp4_urls.is_empty() {
                    let best = mp4_urls.iter().max_by_key(|e| {
                        e.get(VIDEO_DEFINITION_ATTR)
                            .and_then(|v| v.as_str())
                            .map(|s| match s {
                                d if d.contains("1080") => 3,
                                d if d.contains("720") => 2,
                                d if d.contains("480") => 1,
                                _ => 0,
                            })
                            .unwrap_or(0)
                    });

                    if let Some(best_url) = best.and_then(|e| e.get(URL)).and_then(|u| u.as_str()) {
                        return Ok(Some(format!("{}{}", best_url, TYPE_QUERY)));
                    }
                }

                if let Some(first) = playbacks.get(0).and_then(|e| e.get(URL)).and_then(|u| u.as_str()) {
                    return Ok(Some(format!("{}{}", first, TYPE_QUERY)));
                }
            }
        }
    }

    warn!("No JSON playback URLs found for {}", video_page_url);
    Ok(None)
}