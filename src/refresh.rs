use std::{
    os::unix::process::CommandExt,
    process::{Command, Stdio},
};

use anyhow::{Context, Result};
use chrono::{Duration, Utc};

use crate::state::{self, UpdateLockAttempt};

pub fn maybe_refresh(max_age: Duration) -> Result<()> {
    let state_path = state::state_file()?;
    let collected_at = match state::read_state(&state_path) {
        Ok(state) => Some(state.collected_at),
        Err(error) if state::is_missing_state(&error) => None,
        Err(error) => return Err(error),
    };

    if !state::is_stale(collected_at, Utc::now(), max_age) {
        return Ok(());
    }

    spawn_update_if_unlocked()
}

fn spawn_update_if_unlocked() -> Result<()> {
    let lock_path = state::update_lock_file()?;
    match state::try_update_lock(&lock_path)? {
        UpdateLockAttempt::Acquired(lock) => {
            drop(lock);
            spawn_detached_update()
        }
        UpdateLockAttempt::AlreadyHeld => Ok(()),
    }
}

fn spawn_detached_update() -> Result<()> {
    let exe = std::env::current_exe().context("resolve current executable")?;
    Command::new(exe)
        .arg("update")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .process_group(0)
        .spawn()
        .context("spawn detached wabi update")?;

    Ok(())
}
