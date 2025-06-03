use std::fs::{self};
use tempfile::tempdir;
use trailerfin_rust::config::config::AppConfig;
use trailerfin_rust::trailers::scraper::create_or_update_strm_file;

#[test]
fn test_create_strm_file_writes_correct_url() {
    let dir = tempdir().unwrap();
    let folder = dir.path();

    let config = AppConfig {
        scan_path: folder.to_string_lossy().to_string(),
        video_filename: "video1.strm".to_string(),
        should_schedule: false,
        schedule: Some("* * * * *".to_string()),
        user_agent: "TestAgent".to_string(),
        threads: 1,
    };

    let url = "https://example.com/video.mp4";

    create_or_update_strm_file(folder, &config, url).unwrap();

    let written = fs::read_to_string(folder.join("backdrops").join("video1.strm")).unwrap();
    assert_eq!(written, url);
}

#[test]
fn test_overwrite_existing_strm_file() {
    let dir = tempdir().unwrap();
    let folder = dir.path();
    let backdrops = folder.join("backdrops");
    fs::create_dir_all(&backdrops).unwrap();

    let file_path = backdrops.join("video1.strm");
    fs::write(&file_path, "old_url").unwrap();

    let config = AppConfig {
        scan_path: folder.to_string_lossy().to_string(),
        video_filename: "video1.strm".to_string(),
        should_schedule: false,
        schedule: Some("* * * * *".to_string()),
        user_agent: "TestAgent".to_string(),
        threads: 1,
    };

    let new_url = "https://example.com/new_video.mp4";
    create_or_update_strm_file(folder, &config, new_url).unwrap();

    let written = fs::read_to_string(file_path).unwrap();
    assert_eq!(written, new_url);
}