use std::path::{Path, PathBuf};
use std::{env, fmt};

use colored::Colorize;
use serde::de::{self, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};

use crate::error::ConfigError;
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
                        "workspace" => {
                            if workspace.is_some() {
                                return Err(de::Error::duplicate_field("workspace"));
                            }
                            workspace = map.next_value().map(process_template)?;
                        }
                        "description" => {
                            if description.is_some() {
                                return Err(de::Error::duplicate_field("description"));
                            }
                            description = map.next_value().map(process_template)?;
                        }
                        "project" => {
                            if project.is_some() {
                                return Err(de::Error::duplicate_field("project"));
                            }
                            project = map.next_value().map(process_template)?;
                        }
                        "task" => {
                            if task.is_some() {
                                return Err(de::Error::duplicate_field("task"));
                            }
                            task = map.next_value().map(process_template)?;
                        }
                        "tags" => {
                            if tags.is_some() {
                                return Err(de::Error::duplicate_field("tags"));
                            }
                            tags = Some(map.next_value()?).map(|tags: Vec<String>| {
                                tags.into_iter().filter_map(process_template).collect()
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
                            if default.is_some() {
                                return Err(de::Error::duplicate_field("* [default]"));
                            }
                            default = Some(map.next_value()?);
                        }
                        _ => {
                            if configs.iter().any(|(branch, _)| branch == &key) {
                                // TODO: Report duplicate key error can't seem to figure out a way to
                                // return the repeated branch here
                                return Err(de::Error::duplicate_field("branch"));
                            }
                            configs.push((key, map.next_value()?));
                        }
                    }
                }
                Ok(TrackConfig {
                    default: default.unwrap_or(BranchConfig::default()),
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

fn resolve_macro(
    base_dir: &Path,
    instruction: Macro,
) -> Result<String, Box<dyn std::error::Error>> {
    match instruction {
        Macro::Branch => {
            let output = std::process::Command::new("git")
                .arg("rev-parse")
                .arg("--abbrev-ref")
                .arg("HEAD")
                .output()?;
            Ok(String::from_utf8(output.stdout)?.trim().to_string())
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
                    let git_root = PathBuf::from(String::from_utf8(output.stdout)?);
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
                    let git_root = PathBuf::from(String::from_utf8(output.stdout)?);
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
            let output = std::process::Command::new("sh")
                .arg("-c")
                .arg(&command)
                .output();
            match output {
                Ok(output) => {
                    if !output.status.success() {
                        return Err(Box::new(ConfigError::ShellResolution(
                            command,
                            String::from_utf8(output.stderr)?,
                        )));
                    }
                    Ok(String::from_utf8(output.stdout)?.trim().to_string())
                }
                Err(e) => Err(Box::new(ConfigError::ShellResolution(
                    command,
                    e.to_string(),
                ))),
            }
        }
    }
}

fn resolve_token(base_dir: &Path, token: &str) -> Result<String, Box<dyn std::error::Error>> {
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
}
