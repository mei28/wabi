use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use clap::{Parser, Subcommand};

use crate::{
    model::{ProviderState, State},
    providers, refresh, render, state,
};

const DEFAULT_MAX_AGE_SECS: u64 = 120;

#[derive(Debug, Parser)]
#[command(
    name = "wabi",
    about = "Render cached Claude and Codex rate limit usage"
)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Update,
    Status {
        #[arg(long)]
        tmux: bool,
        #[arg(long)]
        no_refresh: bool,
        #[arg(long, default_value_t = DEFAULT_MAX_AGE_SECS)]
        max_age: u64,
    },
    Tick {
        #[arg(long, default_value_t = DEFAULT_MAX_AGE_SECS)]
        max_age: u64,
    },
    Json,
}

pub fn run() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Update => update(),
        Command::Status {
            tmux,
            no_refresh,
            max_age,
        } => status(tmux, no_refresh, max_age),
        Command::Tick { max_age } => tick(max_age),
        Command::Json => json(),
    }
}

fn update() -> Result<()> {
    let lock_path = state::update_lock_file()?;
    let _lock = match state::try_update_lock(&lock_path)? {
        state::UpdateLockAttempt::Acquired(lock) => lock,
        state::UpdateLockAttempt::AlreadyHeld => return Ok(()),
    };

    let path = state::state_file()?;
    let state = State {
        collected_at: Utc::now(),
        claude: provider_result(providers::claude::fetch_state()),
        codex: provider_result(providers::codex::fetch_state()),
    };

    state::write_state(&path, &state)?;
    println!("wrote {}", path.display());
    Ok(())
}

fn status(tmux: bool, no_refresh: bool, max_age: u64) -> Result<()> {
    let path = state::state_file()?;
    if !no_refresh {
        refresh::maybe_refresh(max_age_duration(max_age)?)?;
    }

    match state::read_state(&path) {
        Ok(state) => {
            let color_mode = if tmux {
                render::ColorMode::Tmux
            } else {
                render::ColorMode::Ansi
            };
            println!("{}", render::render_state(&state, Utc::now(), color_mode));
            Ok(())
        }
        Err(error) if state::is_missing_state(&error) => {
            let line = if no_refresh {
                render::render_missing_state(&path)
            } else {
                render::render_refreshing_state()
            };
            println!("{line}");
            Ok(())
        }
        Err(error) => Err(error),
    }
}

fn tick(max_age: u64) -> Result<()> {
    refresh::maybe_refresh(max_age_duration(max_age)?)
}

fn json() -> Result<()> {
    let path = state::state_file()?;
    let state = state::read_state(&path)?;
    println!("{}", serde_json::to_string_pretty(&state)?);
    Ok(())
}

fn provider_result(result: Result<ProviderState>) -> ProviderState {
    match result {
        Ok(state) => state,
        Err(error) => ProviderState::error(error.to_string()),
    }
}

fn max_age_duration(max_age: u64) -> Result<Duration> {
    let seconds = i64::try_from(max_age).context("--max-age is too large")?;
    Ok(Duration::seconds(seconds))
}
