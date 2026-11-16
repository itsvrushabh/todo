use crate::models::TodoItem;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilterStatus {
    All,
    Active,
    Completed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AdaptiveMode {
    #[default]
    Auto,
    DesktopPlanning,
    MobileFocus,
    Calm,
}

impl AdaptiveMode {
    pub fn display_label(&self) -> &'static str {
        match self {
            AdaptiveMode::Auto => "🤖 Auto Layout",
            AdaptiveMode::DesktopPlanning => "🖥️ Desktop Planning",
            AdaptiveMode::MobileFocus => "📱 Mobile Focus",
            AdaptiveMode::Calm => "🌿 Calm Mode",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncStatus {
    LocalOnly,
    Syncing,
    Connected,
    Offline { pending_count: usize },
    Error(String),
}

pub enum SyncResult {
    FetchedTodos(Vec<TodoItem>),
    MutationDrained {
        success: bool,
        error: Option<String>,
    },
    SyncError(String),
    SuccessMessage(String),
}

impl crate::models::PendingMutation {
    pub fn to_request(&self, base_url: &str) -> ehttp::Request {
        let base = base_url.trim_end_matches('/');
        match self {
            crate::models::PendingMutation::Create(create) => {
                let body = serde_json::to_vec(create).unwrap_or_default();
                ehttp::Request::new(
                    ehttp::Method::POST,
                    format!("{base}/api/todos"),
                    &[("Accept", "*/*"), ("Content-Type", "application/json")],
                )
                .with_body(body)
            }
            crate::models::PendingMutation::Update { id, update } => {
                let body = serde_json::to_vec(update).unwrap_or_default();
                ehttp::Request::new(
                    ehttp::Method::PUT,
                    format!("{base}/api/todos/{id}"),
                    &[("Accept", "*/*"), ("Content-Type", "application/json")],
                )
                .with_body(body)
            }
            crate::models::PendingMutation::Delete { id } => ehttp::Request::new(
                ehttp::Method::DELETE,
                format!("{base}/api/todos/{id}"),
                &[("Accept", "*/*")],
            ),
            crate::models::PendingMutation::Toggle { id } => ehttp::Request::new(
                ehttp::Method::PATCH,
                format!("{base}/api/todos/{id}/toggle"),
                &[("Accept", "*/*")],
            ),
            crate::models::PendingMutation::ToggleSubtask { id, subtask_id } => {
                ehttp::Request::new(
                    ehttp::Method::PATCH,
                    format!("{base}/api/todos/{id}/subtasks/{subtask_id}/toggle"),
                    &[("Accept", "*/*")],
                )
            }
        }
    }
}
