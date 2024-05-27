use std::path::{Path, PathBuf};
use std::{env, fmt};

use colored::Colorize;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};

use crate::error::ConfigError;
use crate::models::{Entities, ResultWithDefaultError, TimeEntry};
use crate::utilities;

/// BranchConfig optionally determines workspace, description, project, task,
/// tags, and billable status of a time entry.
/// The fields are optional, and if not specified, the default values will be
/// used. The string fields support templating, which will be replaced with live
/// values on deserialization.
///
/// The templating syntax is `{{<field>}}`. The following fields are supported:
/// - `branch` -- the name of the current branch
/// - `base_dir` -- the directory registered with the config file
/// - `parent_base_dir` -- the base directory of the parent repository
/// - `current_dir` -- the current working directory
/// - `parent_dir` -- the parent directory of the current working directory
/// - `git_root` -- the root directory of the current repository
/// - `parent_git_root` -- the parent directory of the current git repository
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
/// ['my-.+']
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

#[derive(Debug, Serialize, Clone)]
pub struct TrackConfig {
    pub default: BranchConfig,
    pub configs: Vec<(String, BranchConfig)>,
}

const WORKSPACE: &str = "workspace";
const DESCRIPTION: &str = "description";
const PROJECT: &str = "project";
const TASK: &str = "task";
const TAGS: &str = "tags";
const BILLABLE: &str = "billable";

const FIELDS: &[&str] = &[WORKSPACE, DESCRIPTION, PROJECT, TASK, TAGS, BILLABLE];

impl<'de> Deserialize<'de> for BranchConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct BranchConfigVisitor {
            base_dir: PathBuf,
        }

        impl<'de> Visitor<'de> for BranchConfigVisitor {
            type Value = BranchConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct BranchConfig")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let process_template = |value: String| process_config_value(&self.base_dir, value);

                let mut workspace: Option<String> = None;
                let mut description: Option<String> = None;
                let mut project: Option<String> = None;
                let mut task: Option<String> = None;
                let mut tags: Option<Vec<String>> = None;
                let mut billable: Option<bool> = None;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        WORKSPACE => {
                            workspace = map.next_value().map(process_template)?;
                        }
                        DESCRIPTION => {
                            description = map.next_value().map(process_template)?;
                        }
                        PROJECT => {
                            project = map.next_value().map(process_template)?;
                        }
                        TASK => {
                            task = map.next_value().map(process_template)?;
                        }
                        TAGS => {
                            tags = Some(map.next_value()?).map(|tags: Vec<String>| {
                                tags.into_iter().filter_map(process_template).collect()
                            });
                        }
                        BILLABLE => {
                            billable = Some(map.next_value()?);
                        }
                        _ => {
                            return Err(de::Error::unknown_field(&key, FIELDS));
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

        deserializer.deserialize_struct(
            "BranchConfig",
            FIELDS,
            BranchConfigVisitor {
                base_dir: super::locate::TRACKED_PATH
                    .to_owned()
                    .expect("Could not locate tracked path while deserializing config"),
            },
        )
    }
}

impl std::fmt::Display for BranchConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = format!(
            "{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n{}: {}\n",
            WORKSPACE.green(),
            self.workspace
                .as_ref()
                .unwrap_or(&"default".purple().to_string()),
            DESCRIPTION.green(),
            self.description
                .as_ref()
                .unwrap_or(&"none".yellow().to_string()),
            PROJECT.green(),
            self.project
                .as_ref()
                .unwrap_or(&"none".yellow().to_string()),
            TASK.green(),
            self.task
                .as_ref()
                .unwrap_or(&"none".yellow().to_string())
                .trim(),
            TAGS.green(),
            self.tags
                .as_ref()
                .map(|tags| format!("[{}]", tags.join(", ")))
                .unwrap_or("[]".yellow().to_string()),
            BILLABLE.green(),
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
            self.configs
                .iter()
                .map(|(branch, config)| {
                    format!("{}\n{}", format!("[{}]", branch).blue().bold(), config)
                })
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

impl<'de> Deserialize<'de> for TrackConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct TrackConfigVisitor;

        impl<'de> Visitor<'de> for TrackConfigVisitor {
            type Value = TrackConfig;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct TrackConfig")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Self::Value, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut default: Option<BranchConfig> = None;
                let mut configs: Vec<(String, BranchConfig)> = Vec::new();
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "*" => {
                            default = Some(map.next_value()?);
                        }
                        _ => {
                            configs.push((key, map.next_value()?));
                        }
                    }
                }
                Ok(TrackConfig {
                    default: default.unwrap_or_default(),
                    configs,
                })
            }
        }

        const FIELDS: &[&str] = &["default", "branches"];

        deserializer.deserialize_struct("TrackConfig", FIELDS, TrackConfigVisitor)
    }
}

enum Macro {
    Branch,
    BaseDir,
    ParentBaseDir,
    CurrentDir,
    ParentDir,
    GitRoot,
    ParentGitRoot,
    Shell(String),
}

fn resolve_token_to_macro(token: &str) -> Option<Macro> {
    match token {
        "branch" => Some(Macro::Branch),
        "base_dir" => Some(Macro::BaseDir),
        "parent_base_dir" => Some(Macro::ParentBaseDir),
        "current_dir" => Some(Macro::CurrentDir),
        "parent_dir" => Some(Macro::ParentDir),
        "git_root" => Some(Macro::GitRoot),
        "parent_git_root" => Some(Macro::ParentGitRoot),
        _ => {
            if token.starts_with('$') {
                let command = token.trim_start_matches('$').trim();
                Some(Macro::Shell(command.to_string()))
            } else {
                None
            }
        }
    }
}

fn resolve_macro(base_dir: &Path, instruction: Macro) -> ResultWithDefaultError<String> {
    match instruction {
        Macro::Branch => {
            let output = std::process::Command::new("git")
                .arg("rev-parse")
                .arg("--abbrev-ref")
                .arg("HEAD")
                .output()
                .expect("Failed to resolve branch");
            Ok(String::from_utf8(output.stdout)
                .expect("Failed to convert branch name to string. This should never happen.")
                .trim()
                .to_string())
        }
        Macro::BaseDir => Ok(base_dir
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .trim()
            .to_string()),
        Macro::ParentBaseDir => {
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
            let output = env::current_dir().expect("Failed to get current directory");
            Ok(output.file_name().unwrap().to_str().unwrap().to_string())
        }
        Macro::ParentDir => {
            let current_dir = env::current_dir().expect("Failed to get current directory");
            let parent_dir_path = current_dir.parent().unwrap().to_path_buf();
            Ok(parent_dir_path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string())
        }
        Macro::GitRoot => {
            let output = std::process::Command::new("git")
                .arg("rev-parse")
                .arg("--show-toplevel")
                .output();

            match output {
                Ok(output) => {
                    if !output.status.success() {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Failed to resolve git root",
                        )));
                    }
                    let git_root = PathBuf::from(
                        String::from_utf8(output.stdout)
                            .map(|s| s.trim().to_string())
                            .expect(
                                "Failed to convert git root to string. This should never happen.",
                            ),
                    );

                    // Check if we are in a git worktree
                    // If so, we need to get the root of the main repository
                    let git_dir = git_root.join(".git");
                    if git_dir.is_file() {
                        let git_dir = std::fs::read_to_string(git_dir)
                            .expect("Failed to read git directory. No git root maybe?");

                        let git_dir = git_dir
                            .split(':')
                            .nth(1)
                            .expect("Failed to resolve git worktree root")
                            .trim();

                        let git_root = std::fs::canonicalize(git_dir)
                            .map(PathBuf::from)
                            .expect("Failed to canonicalize Git root directory");

                        // git_root stores the path to the main repository in the following format
                        // gitdir: /path/to/main/repo/.git/worktrees/<worktree_name>
                        // We need to extract the name to the main repository `repo`
                        return Ok(git_root
                            .parent()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .file_name()
                            .unwrap()
                            .to_str()
                            .unwrap()
                            .trim()
                            .to_string());
                    }

                    Ok(git_root
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .trim()
                        .to_string())
                }
                Err(err) => Err(Box::new(err)),
            }
        }
        Macro::ParentGitRoot => {
            let output = std::process::Command::new("git")
                .arg("rev-parse")
                .arg("--show-toplevel")
                .output();

            match output {
                Ok(output) => {
                    if !output.status.success() {
                        return Err(Box::new(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "Failed to resolve git root",
                        )));
                    }
                    let git_root = PathBuf::from(
                        String::from_utf8(output.stdout)
                            .expect("Failed to convert git root to string. Not in a git root?"),
                    );
                    Ok(git_root
                        .parent()
                        .unwrap()
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .trim()
                        .to_string())
                }
                Err(err) => Err(Box::new(err)),
            }
        }
        Macro::Shell(command) => {
            let output = utilities::get_shell_cmd(&command).output();
            match output {
                Ok(output) => {
                    if !output.status.success() {
                        return Err(Box::new(ConfigError::ShellResolution(
                            command,
                            String::from_utf8(output.stderr).unwrap(),
                        )));
                    }
                    Ok(String::from_utf8(output.stdout)
                        .expect(
                            "Failed to convert shell output to string. This should never happen.",
                        )
                        .trim()
                        .to_string())
                }
                Err(e) => Err(Box::new(ConfigError::ShellResolution(
                    command,
                    e.to_string(),
                ))),
            }
        }
    }
}

fn resolve_token(base_dir: &Path, token: &str) -> ResultWithDefaultError<String> {
    match resolve_token_to_macro(token) {
        Some(macro_) => resolve_macro(base_dir, macro_),
        None => Err(Box::new(ConfigError::UnrecognizedMarco(token.to_string()))),
    }
}

fn process_config_value(base_dir: &Path, input: String) -> Option<String> {
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
            let resolved = resolve_token(base_dir, &token).map_err(|e| {
                println!("Failed to resolve token: {}", e);
            });
            if let Ok(resolved_token) = resolved {
                result.push_str(&resolved_token);
            } else {
                return None;
            }
        } else {
            result.push(c);
        }
    }

    Some(result)
}

impl TrackConfig {
    fn get_branch_config(&self, branch: Option<&str>) -> &BranchConfig {
        match branch {
            Some(branch) => self
                .configs
                .iter()
                .find(|(b, _)| {
                    let re = regex::Regex::new(b).expect("Invalid branch regex");
                    re.is_match(branch)
                })
                .map(|(_, c)| c)
                .unwrap_or(&self.default),
            None => &self.default,
        }
    }
    pub fn get_branch_config_for_dir(&self, dir: &PathBuf) -> &BranchConfig {
        let branch = utilities::get_git_branch_for_dir(dir);
        self.get_branch_config(branch.as_deref())
    }
    pub fn get_active_config(&self) -> ResultWithDefaultError<&BranchConfig> {
        let current_dir = std::env::current_dir().expect("Failed to get current directory");
        return Ok(self.get_branch_config_for_dir(&current_dir));
    }
    pub fn get_default_entry(&self, entities: Entities) -> ResultWithDefaultError<TimeEntry> {
        let config = self.get_active_config()?;

        let project = config.project.clone().and_then(|name| {
            entities
                .projects
                .clone()
                .into_values()
                .find(|p| p.name == name)
        });

        let project_id = project.clone().map(|p| p.id);

        let task = config.task.clone().and_then(|name| {
            entities
                .tasks
                .clone()
                .into_values()
                .find(|t| t.name == name && t.project.id == project_id.unwrap())
        });

        let time_entry = TimeEntry {
            // TODO: Add support for workspace
            description: config.description.clone().unwrap_or_default(),
            billable: config.billable,
            tags: config.tags.clone().unwrap_or_default(),
            project,
            task,
            ..Default::default()
        };

        Ok(time_entry)
    }
}
