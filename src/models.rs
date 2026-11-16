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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecurrencePattern {
    Daily,
    Weekly,
    Monthly,
    CustomDays(u32),
}

impl fmt::Display for RecurrencePattern {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RecurrencePattern::Daily => write!(f, "daily"),
            RecurrencePattern::Weekly => write!(f, "weekly"),
            RecurrencePattern::Monthly => write!(f, "monthly"),
            RecurrencePattern::CustomDays(days) => write!(f, "every {days} days"),
        }
    }
}

impl FromStr for RecurrencePattern {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim().to_lowercase();
        match trimmed.as_str() {
            "daily" | "day" | "d" => Ok(RecurrencePattern::Daily),
            "weekly" | "week" | "w" => Ok(RecurrencePattern::Weekly),
            "monthly" | "month" | "m" => Ok(RecurrencePattern::Monthly),
            s if s.starts_with("custom:") => {
                let rest = s.trim_start_matches("custom:").trim();
                let days: u32 = rest
                    .parse()
                    .map_err(|_| format!("Invalid custom days: '{rest}'"))?;
                if days == 0 {
                    return Err("Custom days must be at least 1".to_string());
                }
                Ok(RecurrencePattern::CustomDays(days))
            }
            s if s.ends_with("days") || s.ends_with("day") || s.ends_with('d') => {
                let num_str = s
                    .trim_end_matches("days")
                    .trim_end_matches("day")
                    .trim_end_matches('d')
                    .trim();
                let days: u32 = num_str
                    .parse()
                    .map_err(|_| format!("Invalid recurrence format: '{s}'"))?;
                if days == 0 {
                    return Err("Custom days must be at least 1".to_string());
                }
                Ok(RecurrencePattern::CustomDays(days))
            }
            other => Err(format!(
                "Invalid recurrence pattern: '{other}'. Expected 'daily', 'weekly', 'monthly', or 'Nd'."
            )),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TodoItem {
    #[serde(default)]
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
    #[serde(default)]
    pub recurrence: Option<RecurrencePattern>,
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
            recurrence: None,
        }
    }

    pub fn with_due_date(mut self, due_date: Option<String>) -> Self {
        self.due_date = due_date;
        self
    }

    pub fn with_recurrence(mut self, recurrence: Option<RecurrencePattern>) -> Self {
        self.recurrence = recurrence;
        self
    }

    pub fn next_recurrence_date(&self) -> Option<String> {
        let recurrence = self.recurrence.as_ref()?;
        let now = Utc::now();
        let base_date = if let Some(due) = self.due_date.as_deref() {
            let trimmed = due.trim();
            chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d")
                .ok()
                .or_else(|| {
                    chrono::DateTime::parse_from_rfc3339(trimmed)
                        .ok()
                        .map(|dt| dt.date_naive())
                })
                .unwrap_or_else(|| now.date_naive())
        } else {
            now.date_naive()
        };

        let today = now.date_naive();
        let mut next_date = base_date;

        loop {
            next_date = match recurrence {
                RecurrencePattern::Daily => next_date.succ_opt().unwrap_or(next_date),
                RecurrencePattern::Weekly => next_date + chrono::Duration::days(7),
                RecurrencePattern::Monthly => next_date
                    .checked_add_months(chrono::Months::new(1))
                    .unwrap_or_else(|| next_date + chrono::Duration::days(30)),
                RecurrencePattern::CustomDays(days) => {
                    let d = (*days).max(1) as i64;
                    next_date + chrono::Duration::days(d)
                }
            };
            if next_date > today {
                break;
            }
        }
        Some(next_date.format("%Y-%m-%d").to_string())
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
    #[serde(default)]
    pub recurrence: Option<RecurrencePattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TodoUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub completed: Option<bool>,
    pub priority: Option<Priority>,
    pub due_date: Option<String>,
    #[serde(default)]
    pub recurrence: Option<RecurrencePattern>,
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
    fn test_recurrence_pattern_parse_and_display() {
        assert_eq!(
            "daily".parse::<RecurrencePattern>().unwrap(),
            RecurrencePattern::Daily
        );
        assert_eq!(
            "weekly".parse::<RecurrencePattern>().unwrap(),
            RecurrencePattern::Weekly
        );
        assert_eq!(
            "monthly".parse::<RecurrencePattern>().unwrap(),
            RecurrencePattern::Monthly
        );
        assert_eq!(
            "custom:5".parse::<RecurrencePattern>().unwrap(),
            RecurrencePattern::CustomDays(5)
        );
        assert_eq!(
            "3d".parse::<RecurrencePattern>().unwrap(),
            RecurrencePattern::CustomDays(3)
        );
        assert_eq!(
            "10days".parse::<RecurrencePattern>().unwrap(),
            RecurrencePattern::CustomDays(10)
        );

        assert_eq!(RecurrencePattern::Daily.to_string(), "daily");
        assert_eq!(RecurrencePattern::Weekly.to_string(), "weekly");
        assert_eq!(RecurrencePattern::Monthly.to_string(), "monthly");
        assert_eq!(RecurrencePattern::CustomDays(3).to_string(), "every 3 days");
    }

    #[test]
    fn test_next_recurrence_date() {
        let mut item = TodoItem::new(1, "Daily Routine".to_string(), None, Priority::High)
            .with_recurrence(Some(RecurrencePattern::Daily));

        let today = Utc::now().date_naive();
        item.due_date = Some(today.format("%Y-%m-%d").to_string());

        let next = item.next_recurrence_date().unwrap();
        let expected = (today + chrono::Duration::days(1))
            .format("%Y-%m-%d")
            .to_string();
        assert_eq!(next, expected);

        // Weekly
        item.recurrence = Some(RecurrencePattern::Weekly);
        let next_weekly = item.next_recurrence_date().unwrap();
        let expected_weekly = (today + chrono::Duration::days(7))
            .format("%Y-%m-%d")
            .to_string();
        assert_eq!(next_weekly, expected_weekly);

        // Overdue recurring task advances past today
        item.due_date = Some("2020-01-01".to_string());
        item.recurrence = Some(RecurrencePattern::Daily);
        let next_from_overdue = item.next_recurrence_date().unwrap();
        assert_eq!(next_from_overdue, expected);
    }

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
        assert_eq!(item.recurrence, None);
        assert_eq!(item.priority, Priority::Medium);
    }
}
