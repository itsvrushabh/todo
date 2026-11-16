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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DueStatus {
    Overdue,
    DueToday,
    Upcoming,
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
    #[serde(default)]
    pub due_date: Option<String>,
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
            due_date: None,
        }
    }

    pub fn with_due_date(mut self, due_date: Option<String>) -> Self {
        self.due_date = due_date;
        self
    }

    pub fn due_status(&self) -> Option<DueStatus> {
        let due_str = self.due_date.as_deref()?;
        let trimmed = due_str.trim();
        if trimmed.is_empty() {
            return None;
        }

        let due_dt = chrono::DateTime::parse_from_rfc3339(trimmed)
            .ok()
            .map(|dt| dt.with_timezone(&Utc))
            .or_else(|| {
                chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
                    .ok()
                    .map(|d| {
                        d.and_hms_opt(23, 59, 59)
                            .map(|dt| dt.and_utc())
                            .unwrap_or_else(Utc::now)
                    })
            })?;

        let now = Utc::now();
        if due_dt < now {
            Some(DueStatus::Overdue)
        } else if due_dt.date_naive() == now.date_naive() {
            Some(DueStatus::DueToday)
        } else {
            Some(DueStatus::Upcoming)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TodoCreate {
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub priority: Option<Priority>,
    #[serde(default)]
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TodoUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
    pub priority: Option<Priority>,
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TodoStats {
    pub total: usize,
    pub completed: usize,
    pub active: usize,
    pub completion_percentage: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_due_status_calculation() {
        let mut item = TodoItem::new(1, "Task".to_string(), None, Priority::Medium);
        assert_eq!(item.due_status(), None);

        // Overdue (yesterday)
        item.due_date = Some("2020-01-01".to_string());
        assert_eq!(item.due_status(), Some(DueStatus::Overdue));

        // Upcoming (year 2099)
        item.due_date = Some("2099-12-31".to_string());
        assert_eq!(item.due_status(), Some(DueStatus::Upcoming));

        // Today
        let today = Utc::now().format("%Y-%m-%d").to_string();
        item.due_date = Some(today);
        assert_eq!(item.due_status(), Some(DueStatus::DueToday));
    }

    #[test]
    fn test_backward_compatibility_deserialize() {
        let json = r#"{"id":1,"title":"Legacy Task","created_at":"2026-01-01T00:00:00Z"}"#;
        let item: TodoItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.due_date, None);
        assert_eq!(item.priority, Priority::Medium);
    }
}
