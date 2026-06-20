use std::{
    env, fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use thiserror::Error;

use crate::model::State;

#[derive(Debug, Error)]
pub enum StatePathError {
    #[error("HOME is not set; cannot resolve wabi state path")]
    MissingHome,
}

pub fn state_file() -> Result<PathBuf> {
    let state_home = match env::var_os("XDG_STATE_HOME") {
        Some(value) if !value.is_empty() => PathBuf::from(value),
        _ => {
            let home = env::var_os("HOME").ok_or(StatePathError::MissingHome)?;
            PathBuf::from(home).join(".local/state")
        }
    };

    Ok(state_home.join("wabi/state.json"))
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
