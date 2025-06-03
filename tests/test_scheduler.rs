use std::sync::Arc;
use trailerfin_rust::scheduler::schedule::{setup_scheduler, ScanFn};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use trailerfin_rust::config::config::{load_config};
use std::env;
use tempfile::tempdir;

#[tokio::test]
async fn test_scheduler_triggers_scan() {
    static CALLED: AtomicUsize = AtomicUsize::new(0);

    let dir = tempdir().unwrap();

    env::set_var("TRAILERFIN_SCAN_PATH", dir.path().to_str().unwrap());
    env::set_var("TRAILERFIN_VIDEO_FILENAME", "test.strm");
    env::set_var("TRAILERFIN_USER_AGENT", "TestAgent");
    env::set_var("TRAILERFIN_SHOULD_SCHEDULE", "true");
    env::set_var("TRAILERFIN_SCHEDULE", "*/1 * * * * *");

    let config = load_config().expect("Expected config to load");

    let mock_scan: ScanFn = Arc::new(|_cfg| {
        Box::pin(async {
            CALLED.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
    });

    let sched = setup_scheduler(config, Some(mock_scan)).await.unwrap();
    sched.start().await.unwrap();

    tokio::time::sleep(Duration::from_secs(2)).await;

    assert!(CALLED.load(Ordering::SeqCst) >= 1);
}