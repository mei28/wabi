use std::{
    env,
    fs::{self, File, OpenOptions},
    io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use chrono::{DateTime, Duration, Utc};
use fs2::FileExt;
use thiserror::Error;

use crate::model::State;

#[derive(Debug, Error)]
pub enum StatePathError {
    #[error("HOME is not set; cannot resolve wabi state path")]
    MissingHome,
}

pub struct UpdateLock {
    _file: File,
}

pub enum UpdateLockAttempt {
    Acquired(UpdateLock),
    AlreadyHeld,
}

pub fn state_file() -> Result<PathBuf> {
    Ok(wabi_state_dir()?.join("state.json"))
}

pub fn update_lock_file() -> Result<PathBuf> {
    Ok(wabi_state_dir()?.join("update.lock"))
}

pub fn is_stale(
    collected_at: Option<DateTime<Utc>>,
    now: DateTime<Utc>,
    max_age: Duration,
) -> bool {
    match collected_at {
        Some(collected_at) => now.signed_duration_since(collected_at) > max_age,
        None => true,
    }
}

pub fn try_update_lock(path: &Path) -> Result<UpdateLockAttempt> {
    let parent = path
        .parent()
        .with_context(|| format!("update lock path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create update lock directory {}", parent.display()))?;

    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .truncate(false)
        .write(true)
        .open(path)
        .with_context(|| format!("open update lock file {}", path.display()))?;

    match file.try_lock_exclusive() {
        Ok(()) => Ok(UpdateLockAttempt::Acquired(UpdateLock { _file: file })),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
            Ok(UpdateLockAttempt::AlreadyHeld)
        }
        Err(error) => {
            Err(error).with_context(|| format!("lock update lock file {}", path.display()))
        }
    }
}

fn wabi_state_dir() -> Result<PathBuf> {
    Ok(state_home()?.join("wabi"))
}

fn state_home() -> Result<PathBuf> {
    match env::var_os("XDG_STATE_HOME") {
        Some(value) if !value.is_empty() => Ok(PathBuf::from(value)),
        _ => {
            let home = env::var_os("HOME").ok_or(StatePathError::MissingHome)?;
            Ok(PathBuf::from(home).join(".local/state"))
        }
    }
}

pub fn read_state(path: &Path) -> Result<State> {
    let data =
        fs::read_to_string(path).with_context(|| format!("read state file {}", path.display()))?;
    serde_json::from_str(&data).with_context(|| format!("parse state file {}", path.display()))
}

pub fn write_state(path: &Path, state: &State) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("state path has no parent: {}", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("create state directory {}", parent.display()))?;

    let data = serde_json::to_vec_pretty(state).context("serialize state")?;
    fs::write(path, data).with_context(|| format!("write state file {}", path.display()))
}

pub fn is_missing_state(error: &anyhow::Error) -> bool {
    error
        .chain()
        .filter_map(|cause| cause.downcast_ref::<io::Error>())
        .any(|io_error| io_error.kind() == io::ErrorKind::NotFound)
}
