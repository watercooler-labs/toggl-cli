use std::collections::HashMap;
use std::ffi::OsStr;
use std::os::unix::prelude::OsStrExt;
use std::path::PathBuf;
use std::{env, fmt};

use colored::Colorize;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};

/// BranchConfig optionally determines workspace, description, project, task,
/// tags, and billable status of a time entry.
/// The fields are optional, and if not specified, the default values will be
/// used. The string fields support templating, which will be replaced with live
/// values on deserialization.
///
/// The templating syntax is `{{<field>}}`. The following fields are supported:
/// - `branch` -- the name of the current branch
/// - `base_dir` -- the base directory of the repository
/// - `parent_base_dir` -- the base directory of the parent repository
/// - `current_dir` -- the current working directory
/// - `parent_dir` -- the parent directory of the current working directory
/// - `$<shell_statement>` -- the output of the shell statement
///
/// The variables can be combined with other strings,
/// e.g. `{{parent_base_dir}}/{{base_dir}}`.
///
/// The variables can also be used in the middle of a string,
/// e.g. `Working on {{branch}} for {{base_dir}}`.
///
/// Given the following configuration:
/// ```toml
/// [default]
/// workspace = "Default"
/// description = "Working on {{branch}} for {{base_dir}}"
/// project = "Default"
/// task = "Development"
/// tags = ["{{branch}}", "{{$ date +\"%Y\"}}"]
/// billable = true
/// ```
///
/// and the following shell state:
/// ```bash
/// $ git rev-parse --show-toplevel
/// /Users/username/Projects/project
/// $ pwd
/// /Users/username/Projects/project/foo
/// $ git rev-parse --abbrev-ref HEAD
/// my-feature
/// $ date +'%Y'
/// 2023
/// ```
///
/// the following time entry will be created:
/// ```json
/// {
///  "workspace": "Default",
///  "description": "Working on my-feature for project",
///  "project": "Default",
///  "task": "Development",
///  "tags": ["my-feature", "2023"],
///  "billable": true
/// }
/// ```
#[derive(Debug, Serialize, Clone, Default)]
pub struct BranchConfig {
    pub workspace: Option<String>,
    pub description: Option<String>,
    pub project: Option<String>,
    pub task: Option<String>,
    pub tags: Option<Vec<String>>,
    pub billable: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TrackConfig {
    pub default: BranchConfig,
    pub branches: HashMap<String, BranchConfig>,
}

impl<'de> Deserialize<'de> for BranchConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BranchConfigVisitor;

        impl<'de> Visitor<'de> for BranchConfigVisitor {
            type Value = BranchConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BranchConfig")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut workspace: Option<String> = None;
                let mut description: Option<String> = None;
                let mut project: Option<String> = None;
                let mut task: Option<String> = None;
                let mut tags: Option<Vec<String>> = None;
                let mut billable: Option<bool> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "workspace" => {
                            if workspace.is_some() {
                                return Err(de::Error::duplicate_field("workspace"));
                            }
                            workspace = Some(map.next_value().map(process_config_value)?);
                        }
                        "description" => {
                            if description.is_some() {
                                return Err(de::Error::duplicate_field("description"));
                            }
                            description = Some(map.next_value().map(process_config_value)?);
                        }
                        "project" => {
                            if project.is_some() {
                                return Err(de::Error::duplicate_field("project"));
                            }
                            project = Some(map.next_value().map(process_config_value)?);
                        }
                        "task" => {
                            if task.is_some() {
                                return Err(de::Error::duplicate_field("task"));
                            }
                            task = Some(map.next_value().map(process_config_value)?);
                        }
                        "tags" => {
                            if tags.is_some() {
                                return Err(de::Error::duplicate_field("tags"));
                            }
                            tags = Some(map.next_value()?).map(|tags: Vec<String>| {
                                tags.into_iter().map(process_config_value).collect()
                            });
                        }
                        "billable" => {
                            if billable.is_some() {
                                return Err(de::Error::duplicate_field("billable"));
                            }
                            billable = Some(map.next_value()?);
                        }
                        _ => {
                            return Err(de::Error::unknown_field(
                                &key,
                                &[
                                    "workspace",
                                    "description",
                                    "project",
                                    "task",
                                    "tags",
                                    "billable",
                                ],
                            ));
                        }
                    }
                }
                Ok(BranchConfig {
                    workspace,
                    description,
                    project,
                    task,
                    tags,
                    billable: billable.unwrap_or(false),
                })
            }
        }

        const FIELDS: &[&str] = &[
            "workspace",
            "description",
            "project",
            "task",
            "tags",
            "billable",
        ];

        deserializer.deserialize_struct("BranchConfig", FIELDS, BranchConfigVisitor)
    }
}

impl std::fmt::Display for BranchConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = format!(
            "{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n",
            "workspace".green(),
            self.workspace
                .as_ref()
                .unwrap_or(&"default".purple().to_string()),
            "description".green(),
            self.description
                .as_ref()
                .unwrap_or(&"none".yellow().to_string()),
            "project".green(),
            self.project
                .as_ref()
                .unwrap_or(&"none".yellow().to_string()),
            "task".green(),
            self.task
                .as_ref()
                .unwrap_or(&"none".yellow().to_string())
                .trim(),
            "tags".green(),
            self.tags
                .as_ref()
                .map(|tags| format!("[{}]", tags.join(", ")))
                .unwrap_or("[]".yellow().to_string()),
            "billable".green(),
            self.billable,
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

enum Macro {
    Branch,
    BaseDir,
    ParentBaseDir,
    CurrentDir,
    ParentDir,
    Shell(String),
}

fn resolve_token_to_macro(token: &str) -> Option<Macro> {
    match token {
        "branch" => Some(Macro::Branch),
        "base_dir" => Some(Macro::BaseDir),
        "parent_base_dir" => Some(Macro::ParentBaseDir),
        "current_dir" => Some(Macro::CurrentDir),
        "parent_dir" => Some(Macro::ParentDir),
        _ => {
            if token.starts_with('$') {
                let command = token.trim_start_matches('$').trim();
                Some(Macro::Shell(command.to_string()))
            } else {
                println!("{} {}", "Invalid macro".red(), token.red().bold());
                None
            }
        }
    }
}

fn resolve_macro(instruction: Macro) -> Result<String, Box<dyn std::error::Error>> {
    match instruction {
        Macro::Branch => {
            let output = std::process::Command::new("git")
                .arg("rev-parse")
                .arg("--abbrev-ref")
                .arg("HEAD")
                .output()?;
            Ok(String::from_utf8(output.stdout)?.trim().to_string())
        }
        Macro::BaseDir => {
            let base_dir = std::process::Command::new("git")
                .arg("rev-parse")
                .arg("--show-toplevel")
                .output()
                .map(|o| PathBuf::from(OsStr::from_bytes(&o.stdout)))?;
            Ok(base_dir
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
                .trim()
                .to_string())
        }
        Macro::ParentBaseDir => {
            let base_dir = std::process::Command::new("git")
                .arg("rev-parse")
                .arg("--show-toplevel")
                .output()
                .map(|o| PathBuf::from(OsStr::from_bytes(&o.stdout)))?;
            let parent_dir = base_dir
                .parent()
                .unwrap()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            Ok(parent_dir)
        }
        Macro::CurrentDir => {
            let output = env::current_dir()?;
            Ok(output.file_name().unwrap().to_str().unwrap().to_string())
        }
        Macro::ParentDir => {
            let current_dir = env::current_dir()?;
            let parent_dir_path = current_dir.parent().unwrap().to_path_buf();
            Ok(parent_dir_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string())
        }
        Macro::Shell(command) => {
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&command)
                .output();
            match output {
                Ok(output) => {
                    if !output.status.success() {
                        println!(
                            "{}: {}\n{}: {}",
                            "Error resolving shell macro".red(),
                            String::from_utf8(output.stderr)?.red().bold(),
                            "Command".yellow(),
                            command.yellow().bold()
                        );
                        return Ok(command);
                    }
                    Ok(String::from_utf8(output.stdout)?.trim().to_string())
                }
                Err(e) => {
                    println!(
                        "{}: {}\n{}: {}",
                        "Error resolving shell macro".red(),
                        e.to_string().red().bold(),
                        "Command".yellow(),
                        command.yellow().bold()
                    );
                    Ok(command)
                }
            }
        }
    }
}

fn resolve_token(token: &str) -> Result<String, Box<dyn std::error::Error>> {
    match resolve_token_to_macro(token) {
        Some(macro_) => resolve_macro(macro_),
        None => Ok(token.to_string()),
    }
}

fn process_config_value(input: String) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' && chars.peek() == Some(&'{') {
            chars.next();
            let mut token = String::new();
            while let Some(c) = chars.next() {
                if c == '}' && chars.peek() == Some(&'}') {
                    chars.next();
                    break;
                }
                token.push(c);
            }
            result.push_str(&resolve_token(&token).unwrap());
        } else {
            result.push(c);
        }
    }

    result
}
