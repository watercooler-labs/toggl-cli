use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct NetworkTimeEntry {
    pub id: i64,
    pub description: String,
    pub start: DateTime<Utc>,
    pub stop: Option<DateTime<Utc>>,
    pub duration: i64,
    pub billable: bool,
    pub workspace_id: i64,
    pub tags: Vec<String>,
    pub project_id: Option<i64>,
    pub task_id: Option<i64>,
    pub created_with: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
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
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NetworkClient {
    pub id: i64,
    pub name: String,
    pub wid: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct NetworkTask {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
}
