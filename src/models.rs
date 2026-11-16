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
    #[serde(default)]
    pub estimated_minutes: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeBudgetFilter {
    #[default]
    All,
    QuickWins,
    ShortSession,
    DeepWork,
}

impl TimeBudgetFilter {
    pub fn matches(&self, minutes: u32) -> bool {
        match self {
            TimeBudgetFilter::All => true,
            TimeBudgetFilter::QuickWins => minutes <= 15,
            TimeBudgetFilter::ShortSession => minutes <= 30,
            TimeBudgetFilter::DeepWork => minutes > 30,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            TimeBudgetFilter::All => "All Durations",
            TimeBudgetFilter::QuickWins => "⚡ Quick Wins (≤15m)",
            TimeBudgetFilter::ShortSession => "⏱️ Short (≤30m)",
            TimeBudgetFilter::DeepWork => "🎯 Deep Work (>30m)",
        }
    }
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
            estimated_minutes: None,
        }
    }

    pub fn parse_duration_str(s: &str) -> Option<u32> {
        let lower = s.to_lowercase();
        for word in lower.split_whitespace() {
            let clean = word.trim_matches(|c: char| !c.is_alphanumeric() && c != '~');
            let clean = clean.trim_start_matches('~');
            if clean.ends_with("mins") || clean.ends_with("min") {
                let num = clean.trim_end_matches("mins").trim_end_matches("min");
                if let Ok(m) = num.parse::<u32>() {
                    return Some(m);
                }
            } else if clean.ends_with('m') {
                let num = clean.trim_end_matches('m');
                if let Ok(m) = num.parse::<u32>() {
                    return Some(m);
                }
            } else if clean.ends_with("hrs") || clean.ends_with("hr") || clean.ends_with('h') {
                let num = clean
                    .trim_end_matches("hrs")
                    .trim_end_matches("hr")
                    .trim_end_matches('h');
                if let Ok(h) = num.parse::<u32>() {
                    return Some(h * 60);
                }
            }
        }
        None
    }

    pub fn effective_estimated_minutes(&self) -> u32 {
        if let Some(m) = self.estimated_minutes {
            return m;
        }
        if let Some(m) = Self::parse_duration_str(&self.title) {
            return m;
        }
        if let Some(ref desc) = self.description {
            if let Some(m) = Self::parse_duration_str(desc) {
                return m;
            }
        }
        match self.priority {
            Priority::Low => 15,
            Priority::Medium => 30,
            Priority::High => 60,
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
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
#[serde(rename_all = "snake_case")]
pub enum PendingMutation {
    Create(TodoCreate),
    Update { id: i64, update: TodoUpdate },
    Delete { id: i64 },
    Toggle { id: i64 },
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
    fn test_parse_duration_and_time_budget() {
        assert_eq!(TodoItem::parse_duration_str("Fix typo 15m"), Some(15));
        assert_eq!(
            TodoItem::parse_duration_str("Refactor auth [30min]"),
            Some(30)
        );
        assert_eq!(TodoItem::parse_duration_str("Deep focus (~1h)"), Some(60));
        assert_eq!(
            TodoItem::parse_duration_str("Long workshop [2hrs]"),
            Some(120)
        );
        assert_eq!(TodoItem::parse_duration_str("Plain task title"), None);

        // Priority fallback heuristic
        let low_task = TodoItem::new(1, "Low task".into(), None, Priority::Low);
        assert_eq!(low_task.effective_estimated_minutes(), 15);

        let med_task = TodoItem::new(2, "Med task".into(), None, Priority::Medium);
        assert_eq!(med_task.effective_estimated_minutes(), 30);

        let high_task = TodoItem::new(3, "High task".into(), None, Priority::High);
        assert_eq!(high_task.effective_estimated_minutes(), 60);

        // Explicit duration overrides priority
        let mut custom = TodoItem::new(4, "High task quick fix".into(), None, Priority::High);
        custom.estimated_minutes = Some(10);
        assert_eq!(custom.effective_estimated_minutes(), 10);

        // TimeBudgetFilter matching
        assert!(TimeBudgetFilter::QuickWins.matches(15));
        assert!(!TimeBudgetFilter::QuickWins.matches(16));
        assert!(TimeBudgetFilter::ShortSession.matches(30));
        assert!(!TimeBudgetFilter::ShortSession.matches(31));
        assert!(TimeBudgetFilter::DeepWork.matches(45));
        assert!(!TimeBudgetFilter::DeepWork.matches(25));
        assert!(TimeBudgetFilter::All.matches(120));
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

    #[test]
    fn test_pending_mutation_serialization() {
        let create_mut = PendingMutation::Create(TodoCreate {
            title: "Offline task".to_string(),
            description: Some("Offline desc".to_string()),
            priority: Some(Priority::High),
            due_date: Some("2026-11-20".to_string()),
            recurrence: Some(RecurrencePattern::Daily),
        });
        let json = serde_json::to_string(&create_mut).unwrap();
        let decoded: PendingMutation = serde_json::from_str(&json).unwrap();
        assert_eq!(create_mut, decoded);

        let toggle_mut = PendingMutation::Toggle { id: 42 };
        let toggle_json = serde_json::to_string(&toggle_mut).unwrap();
        let toggle_decoded: PendingMutation = serde_json::from_str(&toggle_json).unwrap();
        assert_eq!(toggle_mut, toggle_decoded);

        let del_mut = PendingMutation::Delete { id: 7 };
        let del_json = serde_json::to_string(&del_mut).unwrap();
        let del_decoded: PendingMutation = serde_json::from_str(&del_json).unwrap();
        assert_eq!(del_mut, del_decoded);

        let update_mut = PendingMutation::Update {
            id: 10,
            update: TodoUpdate {
                title: Some("Updated title".to_string()),
                description: None,
                completed: Some(true),
                priority: Some(Priority::Low),
                due_date: None,
                recurrence: None,
            },
        };
        let update_json = serde_json::to_string(&update_mut).unwrap();
        let update_decoded: PendingMutation = serde_json::from_str(&update_json).unwrap();
        assert_eq!(update_mut, update_decoded);
    }
}
