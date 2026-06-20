use anyhow::Result;
use chrono::Utc;
use clap::{Parser, Subcommand};

use crate::{
    model::{ProviderState, State},
    providers, render, state,
};

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
    },
    Json,
}

pub fn run() -> Result<()> {
    let args = Args::parse();

    match args.command {
        Command::Update => update(),
        Command::Status { tmux } => status(tmux),
        Command::Json => json(),
    }
}

fn update() -> Result<()> {
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

fn status(tmux: bool) -> Result<()> {
    let path = state::state_file()?;
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
            println!("{}", render::render_missing_state(&path));
            Ok(())
        }
        Err(error) => Err(error),
    }
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
