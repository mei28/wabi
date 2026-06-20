use chrono::{DateTime, Duration, Utc};
use owo_colors::OwoColorize;

use crate::model::{ProviderState, State, Window};

pub const BAR_WIDTH: usize = 14;
pub const STALE_AFTER: Duration = Duration::minutes(15);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorLevel {
    Green,
    Yellow,
    Red,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Plain,
    Ansi,
    Tmux,
}

pub fn bar_cells(used_percentage: f64, width: usize) -> String {
    assert!(
        used_percentage.is_finite(),
        "used_percentage must be finite"
    );
    assert!(
        (0.0..=100.0).contains(&used_percentage),
        "used_percentage must be between 0 and 100"
    );
    assert!(width > 0, "bar width must be positive");

    let filled = ((used_percentage / 100.0) * width as f64).round() as usize;
    format!("{}{}", "█".repeat(filled), "░".repeat(width - filled))
}

pub fn color_level(used_percentage: f64) -> ColorLevel {
    if used_percentage >= 90.0 {
        ColorLevel::Red
    } else if used_percentage >= 70.0 {
        ColorLevel::Yellow
    } else {
        ColorLevel::Green
    }
}

pub fn format_reset_delta(now: DateTime<Utc>, resets_at: DateTime<Utc>) -> String {
    let remaining = resets_at.signed_duration_since(now);
    if remaining <= Duration::zero() {
        return "0m".to_string();
    }

    let total_minutes = remaining.num_minutes();
    let hours = total_minutes / 60;
    let minutes = total_minutes % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}

pub fn render_window(window: &Window, now: DateTime<Utc>, color_mode: ColorMode) -> String {
    let rounded_pct = window.used_percentage.round() as i64;
    let pct = colorize_percentage(rounded_pct, color_level(window.used_percentage), color_mode);
    let bar = bar_cells(window.used_percentage, BAR_WIDTH);

    match window.resets_at {
        Some(resets_at) => {
            let reset_delta = format_reset_delta(now, resets_at);
            let reset_time = resets_at.format("%H:%M");
            format!(
                "{} {} {} {} → {}",
                window.label, bar, pct, reset_delta, reset_time
            )
        }
        None => format!("{} {} {} reset ?", window.label, bar, pct),
    }
}

pub fn render_state(state: &State, now: DateTime<Utc>, color_mode: ColorMode) -> String {
    let mut line = format!(
        "{} | {}",
        render_provider("claude", &state.claude, now, color_mode),
        render_provider("codex", &state.codex, now, color_mode)
    );

    if now.signed_duration_since(state.collected_at) > STALE_AFTER {
        line.push_str(" | stale");
    }

    line
}

pub fn render_missing_state(path: &std::path::Path) -> String {
    format!("wabi: no state at {}; run wabi update", path.display())
}

pub fn render_refreshing_state() -> String {
    "wabi: refreshing... (run in progress)".to_string()
}

fn render_provider(
    name: &str,
    state: &ProviderState,
    now: DateTime<Utc>,
    color_mode: ColorMode,
) -> String {
    if let Some(error) = &state.error {
        return format!("{name} error: {error}");
    }

    let mut windows = Vec::new();
    if let Some(window) = &state.five_hour {
        windows.push(render_window(window, now, color_mode));
    }
    if let Some(window) = &state.secondary {
        windows.push(render_window(window, now, color_mode));
    }

    if windows.is_empty() {
        format!("{name} --")
    } else {
        format!("{name} {}", windows.join(" / "))
    }
}

fn colorize_percentage(rounded_pct: i64, level: ColorLevel, color_mode: ColorMode) -> String {
    let text = format!("{rounded_pct}%");
    match color_mode {
        ColorMode::Plain => text,
        ColorMode::Ansi => match level {
            ColorLevel::Green => text.green().to_string(),
            ColorLevel::Yellow => text.yellow().to_string(),
            ColorLevel::Red => text.red().to_string(),
        },
        ColorMode::Tmux => {
            let color = match level {
                ColorLevel::Green => "green",
                ColorLevel::Yellow => "yellow",
                ColorLevel::Red => "red",
            };
            format!("#[fg={color}]{text}#[default]")
        }
    }
}
