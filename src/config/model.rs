use std::collections::HashMap;

use colored::Colorize;
use serde::{Deserialize, Serialize};

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

impl std::fmt::Display for BranchConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = format!(
            "{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: [{}]\n{}: {}\n",
            "workspace".green(),
            self.workspace.as_ref().unwrap_or(&"Default".to_string()),
            "description".green(),
            self.description.as_ref().unwrap_or(&"None".to_string()),
            "project".green(),
            self.project.as_ref().unwrap_or(&"None".to_string()),
            "task".green(),
            self.task.as_ref().unwrap_or(&"None".to_string()).trim(),
            "tags".green(),
            self.tags
                .as_ref()
                .map(|tags| tags.join(", "))
                .unwrap_or("None".to_string()),
            "billable".green(),
            self.billable
                .as_ref()
                .map(|billable| billable.to_string())
                .unwrap_or("None".to_string()),
        );
        write!(f, "{}", summary)
    }
}

impl std::fmt::Display for TrackConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = format!("{}\n{}", "[default]".purple().bold(), self.default);
        write!(
            f,
            "{}\n{}",
            summary,
            self.branches
                .iter()
                .map(|(branch, config)| {
                    format!("{}\n{}", format!("[{}]", branch).blue().bold(), config)
                })
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}
