use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::TimeEntry;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkTimeEntry {
    pub id: i64,
    pub description: String,
    pub start: DateTime<Utc>,
    pub stop: Option<DateTime<Utc>>,
    pub duration: i64,
    pub billable: bool,
    pub workspace_id: i64,
    pub tags: Option<Vec<String>>,
    pub project_id: Option<i64>,
    pub task_id: Option<i64>,
    pub created_with: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkProject {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
    pub client_id: Option<i64>,
    pub is_private: bool,
    pub active: bool,
    pub at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub server_deleted_at: Option<DateTime<Utc>>,
    pub color: String,
    pub billable: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkClient {
    pub id: i64,
    pub name: String,
    pub wid: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkTask {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
    pub project_id: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NetworkWorkspace {
    pub id: i64,
    pub name: String,
    pub admin: bool,
}

impl From<TimeEntry> for NetworkTimeEntry {
    fn from(value: TimeEntry) -> Self {
        NetworkTimeEntry {
            id: value.id,
            description: value.description.to_string(),
            start: value.start,
            stop: value.stop,
            duration: value.duration,
            billable: value.billable,
            workspace_id: value.workspace_id,
            tags: if value.tags.is_empty() { None } else { Some(value.tags.clone()) },
            project_id: value.project.map(|p| p.id),
            task_id: value.task.map(|t| t.id),
            created_with: value.created_with,
        }
    }
}
