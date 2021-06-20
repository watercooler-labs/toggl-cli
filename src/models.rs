use chrono::{DateTime, Duration, Utc};
use colored::Colorize;
use serde::{Deserialize, Serialize};

pub type ResultWithDefaultError<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub api_token: String,
    pub email: String,
    pub fullname: Option<String>,
    pub timezone: String,
    pub default_workspace_id: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TimeEntry {
    pub id: i64,
    pub description: String,
    pub start: DateTime<Utc>,
    pub stop: Option<DateTime<Utc>>,
    pub duration: i64,
    pub billable: bool,
    pub workspace_id: i64,
    pub project_id: Option<i64>,
    pub task_id: Option<i64>,
    pub created_with: Option<String>,
}

impl TimeEntry {
    pub fn get_description(&self) -> String {
        match self.description.as_ref() {
            "" => "(no description)".to_string(),
            _ => self.description.to_string(),
        }
    }

    pub fn get_duration(&self) -> Duration {
        match self.stop {
            Some(_) => Duration::seconds(self.duration),
            None => Utc::now().signed_duration_since(self.start),
        }
    }

    pub fn get_duration_hmmss(&self) -> String {
        let duration = self.get_duration();
        return format!(
            "{}:{:02}:{:02}",
            duration.num_hours(),
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        );
    }

    pub fn is_running(&self) -> bool {
        self.duration.is_negative()
    }
}

impl std::fmt::Display for TimeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = format!(
            "[{}]{} –  {}",
            if self.is_running() {
                self.get_duration_hmmss().green().bold()
            } else {
                self.get_duration_hmmss().normal()
            },
            if self.is_running() { "*" } else { " " },
            self.get_description()
        );
        write!(f, "{}", summary)
    }
}
