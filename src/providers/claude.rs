use std::process::Command;

use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::model::{ProviderState, Window};

pub const USAGE_ENDPOINT: &str = "https://api.anthropic.com/api/oauth/usage";
pub const BETA_HEADER_VALUE: &str = "oauth-2025-04-20";
const KEYCHAIN_SERVICE: &str = "Claude Code-credentials";

#[derive(Debug, Deserialize)]
struct ClaudeUsageResponse {
    five_hour: ClaudeWindow,
    seven_day: ClaudeWindow,
}

#[derive(Debug, Deserialize)]
struct ClaudeWindow {
    utilization: f64,
    resets_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct ClaudeCredentials {
    #[serde(rename = "claudeAiOauth")]
    claude_ai_oauth: ClaudeOauth,
}

#[derive(Debug, Deserialize)]
struct ClaudeOauth {
    #[serde(rename = "accessToken")]
    access_token: String,
    #[serde(rename = "expiresAt")]
    expires_at: i64,
}

pub fn fetch_state() -> Result<ProviderState> {
    let access_token = read_access_token()?;
    let mut response = match ureq::get(USAGE_ENDPOINT)
        .header("anthropic-beta", BETA_HEADER_VALUE)
        .header("authorization", format!("Bearer {access_token}"))
        .call()
    {
        Ok(response) => response,
        Err(ureq::Error::StatusCode(401)) => {
            return Err(anyhow!(
                "Claude token expired or unauthorized (HTTP 401); refresh via Claude Code"
            ));
        }
        Err(ureq::Error::StatusCode(code)) => {
            return Err(anyhow!("Claude usage request failed (HTTP {code})"));
        }
        Err(error) => return Err(anyhow!("request Claude usage: {error}")),
    };

    let body = response
        .body_mut()
        .read_to_string()
        .context("read Claude usage response body")?;
    parse_usage(&body)
}

pub fn parse_usage(input: &str) -> Result<ProviderState> {
    let response: ClaudeUsageResponse =
        serde_json::from_str(input).context("parse Claude usage response")?;

    Ok(ProviderState::available(
        response.five_hour.into_window("5h"),
        response.seven_day.into_window("7d"),
    ))
}

fn read_access_token() -> Result<String> {
    let output = Command::new("security")
        .args(["find-generic-password", "-s", KEYCHAIN_SERVICE, "-w"])
        .output()
        .context("read Claude credentials from macOS keychain")?;

    if !output.status.success() {
        let stderr =
            String::from_utf8(output.stderr).context("decode security command stderr as UTF-8")?;
        return Err(anyhow!(
            "read Claude credentials from macOS keychain failed: {}",
            stderr.trim()
        ));
    }

    let raw =
        String::from_utf8(output.stdout).context("decode Claude keychain credential as UTF-8")?;
    let credentials: ClaudeCredentials =
        serde_json::from_str(raw.trim()).context("parse Claude keychain credential JSON")?;

    if credentials.claude_ai_oauth.expires_at <= Utc::now().timestamp_millis() {
        return Err(anyhow!(
            "Claude token expired or unauthorized (HTTP 401); refresh via Claude Code"
        ));
    }

    Ok(credentials.claude_ai_oauth.access_token)
}

impl ClaudeWindow {
    fn into_window(self, label: &str) -> Window {
        Window {
            label: label.to_string(),
            used_percentage: self.utilization,
            resets_at: Some(self.resets_at),
        }
    }
}
