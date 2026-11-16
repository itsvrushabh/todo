use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    #[default]
    Medium,
    High,
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::Low => write!(f, "low"),
            Priority::Medium => write!(f, "medium"),
            Priority::High => write!(f, "high"),
        }
    }
}

impl FromStr for Priority {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" | "med" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            other => Err(format!(
                "Invalid priority: '{other}'. Expected 'low', 'medium', or 'high'."
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TodoItem {
    pub id: i64,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub completed: bool,
    pub created_at: String,
    #[serde(default)]
    pub updated_at: Option<String>,
}

impl TodoItem {
    pub fn new(id: i64, title: String, description: Option<String>, priority: Priority) -> Self {
        let now = Utc::now().to_rfc3339();
        Self {
            id,
            title,
            description,
            priority,
            completed: false,
            created_at: now,
            updated_at: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodoCreate {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<Priority>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TodoUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
    pub priority: Option<Priority>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TodoStats {
    pub total: usize,
    pub completed: usize,
    pub active: usize,
    pub completion_percentage: f64,
}
