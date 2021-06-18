use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub api_token: String,
    pub email: String,
    pub fullname: Option<String>,
    pub timezone: String,
    pub default_workspace_id: i64
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
}