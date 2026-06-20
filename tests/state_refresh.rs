use std::{fs, process};

use chrono::{Duration, TimeZone, Utc};
use wabi::state::{is_stale, try_update_lock, UpdateLockAttempt};

#[test]
fn treats_missing_cache_as_stale() {
    let now = Utc.with_ymd_and_hms(2026, 6, 20, 12, 0, 0).unwrap();

    assert!(is_stale(None, now, Duration::seconds(120)));
}

#[test]
fn treats_cache_younger_than_max_age_as_fresh() {
    let now = Utc.with_ymd_and_hms(2026, 6, 20, 12, 0, 0).unwrap();
    let collected_at = now - Duration::seconds(119);

    assert!(!is_stale(Some(collected_at), now, Duration::seconds(120)));
}

#[test]
fn treats_cache_exactly_at_max_age_as_fresh() {
    let now = Utc.with_ymd_and_hms(2026, 6, 20, 12, 0, 0).unwrap();
    let collected_at = now - Duration::seconds(120);

    assert!(!is_stale(Some(collected_at), now, Duration::seconds(120)));
}

#[test]
fn treats_cache_older_than_max_age_as_stale() {
    let now = Utc.with_ymd_and_hms(2026, 6, 20, 12, 0, 0).unwrap();
    let collected_at = now - Duration::seconds(121);

    assert!(is_stale(Some(collected_at), now, Duration::seconds(120)));
}

#[test]
fn fails_to_take_update_lock_twice() {
    let directory = unique_temp_directory();
    fs::create_dir_all(&directory).unwrap();
    let lock_path = directory.join("update.lock");

    let _first_lock = match try_update_lock(&lock_path).unwrap() {
        UpdateLockAttempt::Acquired(lock) => lock,
        UpdateLockAttempt::AlreadyHeld => panic!("first lock acquisition should succeed"),
    };

    assert!(matches!(
        try_update_lock(&lock_path).unwrap(),
        UpdateLockAttempt::AlreadyHeld
    ));

    fs::remove_dir_all(directory).unwrap();
}

fn unique_temp_directory() -> std::path::PathBuf {
    let name = format!(
        "wabi-lock-test-{}-{}",
        process::id(),
        Utc::now().timestamp_nanos_opt().unwrap()
    );
    std::env::temp_dir().join(name)
}
