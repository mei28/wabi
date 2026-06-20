use chrono::{DateTime, Utc};

use wabi::providers::{claude, codex};

#[test]
fn parses_claude_usage_fixture() {
    let provider = claude::parse_usage(include_str!("fixtures/claude_usage.json")).unwrap();

    let five_hour = provider.five_hour.unwrap();
    assert_eq!(five_hour.label, "5h");
    assert_eq!(five_hour.used_percentage, 22.0);
    assert_eq!(
        five_hour.resets_at,
        Some(rfc3339("2026-06-20T10:09:59.871382+00:00"))
    );

    let secondary = provider.secondary.unwrap();
    assert_eq!(secondary.label, "7d");
    assert_eq!(secondary.used_percentage, 13.0);
    assert_eq!(
        secondary.resets_at,
        Some(rfc3339("2026-06-21T06:59:59.871405+00:00"))
    );
    assert!(provider.error.is_none());
}

#[test]
fn parses_codex_rate_limits_fixture() {
    let provider =
        codex::parse_rate_limits(include_str!("fixtures/codex_rate_limits.json")).unwrap();

    let five_hour = provider.five_hour.unwrap();
    assert_eq!(five_hour.label, "5h");
    assert_eq!(five_hour.used_percentage, 1.0);
    assert_eq!(five_hour.resets_at, Some(rfc3339("2026-06-20T10:28:44Z")));

    let secondary = provider.secondary.unwrap();
    assert_eq!(secondary.label, "wk");
    assert_eq!(secondary.used_percentage, 6.0);
    assert_eq!(secondary.resets_at, Some(rfc3339("2026-06-26T14:55:11Z")));
    assert!(provider.error.is_none());
}

#[test]
fn parses_codex_null_resets_at_as_unknown() {
    let provider = codex::parse_rate_limits(
        r#"{
          "id": 2,
          "result": {
            "rateLimits": {
              "primary": { "usedPercent": 1, "resetsAt": null },
              "secondary": { "usedPercent": 6, "resetsAt": null }
            }
          }
        }"#,
    )
    .unwrap();

    assert_eq!(provider.five_hour.unwrap().resets_at, None);
    assert_eq!(provider.secondary.unwrap().resets_at, None);
}

fn rfc3339(input: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(input)
        .unwrap()
        .with_timezone(&Utc)
}
