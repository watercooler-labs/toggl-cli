use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub mod get;
pub mod locate;
pub mod parser;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct BranchConfig {
    pub workspace: Option<String>,
    pub description: Option<String>,
    pub project: Option<String>,
    pub task: Option<String>,
    pub tags: Option<Vec<String>>,
    pub billable: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackConfig {
    pub default: BranchConfig,
    pub branches: HashMap<String, BranchConfig>,
}
