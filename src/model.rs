use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Window {
    pub label: String,
    pub used_percentage: f64,
    pub resets_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProviderState {
    pub five_hour: Option<Window>,
    pub secondary: Option<Window>,
    pub error: Option<String>,
}

impl ProviderState {
    pub fn available(five_hour: Window, secondary: Window) -> Self {
        Self {
            five_hour: Some(five_hour),
            secondary: Some(secondary),
            error: None,
        }
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self {
            five_hour: None,
            secondary: None,
            error: Some(message.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct State {
    pub collected_at: DateTime<Utc>,
    pub claude: ProviderState,
    pub codex: ProviderState,
}
