use chrono::{TimeZone, Utc};

use wabi::model::Window;
use wabi::render::{
    bar_cells, color_level, format_reset_delta, render_window, ColorLevel, ColorMode,
};

#[test]
fn fills_fourteen_cell_bar_by_rounded_usage() {
    assert_eq!(bar_cells(73.2, 14), "██████████░░░░");
    assert_eq!(bar_cells(0.0, 14), "░░░░░░░░░░░░░░");
    assert_eq!(bar_cells(100.0, 14), "██████████████");
}

#[test]
fn classifies_color_thresholds_like_statusline() {
    assert_eq!(color_level(69.9), ColorLevel::Green);
    assert_eq!(color_level(70.0), ColorLevel::Yellow);
    assert_eq!(color_level(89.9), ColorLevel::Yellow);
    assert_eq!(color_level(90.0), ColorLevel::Red);
}

#[test]
fn formats_remaining_reset_delta() {
    let now = Utc.with_ymd_and_hms(2026, 6, 20, 1, 12, 0).unwrap();
    let resets_at = Utc.with_ymd_and_hms(2026, 6, 20, 2, 30, 0).unwrap();

    assert_eq!(format_reset_delta(now, resets_at), "1h 18m");
}

#[test]
fn renders_plain_window_line() {
    let now = Utc.with_ymd_and_hms(2026, 6, 20, 1, 12, 0).unwrap();
    let window = Window {
        label: "5h".to_string(),
        used_percentage: 73.2,
        resets_at: Some(Utc.with_ymd_and_hms(2026, 6, 20, 2, 30, 0).unwrap()),
    };

    assert_eq!(
        render_window(&window, now, ColorMode::Plain),
        "5h ██████████░░░░ 73% 1h 18m → 02:30"
    );
}
