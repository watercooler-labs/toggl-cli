use std::{cmp, env};

use crate::{constants, parcel::Parcel};
use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use colored::{ColoredString, Colorize};
use colors_transform::{Color, Rgb};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

pub type ResultWithDefaultError<T> = Result<T, Box<dyn std::error::Error + Send>>;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
}

impl std::fmt::Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Entities {
    pub time_entries: Vec<TimeEntry>,
    pub projects: HashMap<i64, Project>,
    pub tasks: HashMap<i64, Task>,
    pub clients: HashMap<i64, Client>,
    pub workspaces: Vec<Workspace>,
    pub tags: Vec<Tag>,
}

impl Entities {
    pub fn running_time_entry(&self) -> Option<TimeEntry> {
        self.time_entries.iter().find(|te| te.is_running()).cloned()
    }

    pub fn workspace_id_for_name(&self, name: &str) -> Option<i64> {
        self.workspaces
            .iter()
            .find(|w| w.name == name)
            .map(|w| w.id)
    }

    pub fn project_for_name(&self, workspace_id: i64, name: &str) -> Option<Project> {
        self.projects
            .values()
            .find(|p| p.workspace_id == workspace_id && p.name == name)
            .cloned()
    }

    pub fn task_for_name(&self, workspace_id: i64, project_id: i64, name: &str) -> Option<Task> {
        self.tasks
            .values()
            .find(|t| {
                t.workspace_id == workspace_id && t.project.id == project_id && t.name == name
            })
            .cloned()
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct User {
    pub api_token: String,
    pub email: String,
    pub fullname: Option<String>,
    pub timezone: String,
    pub default_workspace_id: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TimeEntry {
    pub id: i64,
    pub description: String,
    pub start: DateTime<Utc>,
    pub stop: Option<DateTime<Utc>>,
    pub duration: i64,
    pub billable: bool,
    pub workspace_id: i64,
    pub tags: Vec<String>,
    pub project: Option<Project>,
    pub task: Option<Task>,
    pub created_with: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
    pub client: Option<Client>,
    pub is_private: bool,
    pub active: bool,
    pub at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub color: String,
    pub billable: Option<bool>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Workspace {
    pub id: i64,
    pub name: String,
    pub admin: bool,
}

lazy_static! {
    pub static ref HAS_TRUECOLOR_SUPPORT: bool = if let Ok(truecolor) = env::var("COLORTERM") {
        truecolor == "truecolor" || truecolor == "24bit"
    } else {
        false
    };
}

impl Project {
    fn text_in_closest_terminal_color(
        &self,
        text: &str,
        red: u8,
        green: u8,
        blue: u8,
    ) -> ColoredString {
        let colors = vec![
            (0, 0, 0),       //Black
            (205, 0, 0),     //Red
            (0, 205, 0),     //Green
            (205, 205, 0),   //Yellow
            (0, 0, 238),     //Blue
            (205, 0, 205),   //Magenta
            (0, 205, 205),   //Cyan
            (229, 229, 229), //White
            (127, 127, 127), //BrightBlack
            (255, 0, 0),     //BrightRed
            (0, 255, 0),     //BrightGreen
            (255, 255, 0),   //BrightYellow
            (92, 92, 255),   //BrightBlue
            (255, 0, 255),   //BrightMagenta
            (0, 255, 255),   //BrightCyan
            (255, 255, 255), //BrightWhite
        ]
        .into_iter()
        .enumerate();

        let index = colors
            .map(|(index, (r, g, b))| {
                let rd = cmp::max(r, red) - cmp::min(r, red);
                let gd = cmp::max(g, green) - cmp::min(g, green);
                let bd = cmp::max(b, blue) - cmp::min(b, blue);
                let rd: u32 = rd.into();
                let gd: u32 = gd.into();
                let bd: u32 = bd.into();
                let distance: u32 = rd.pow(2) + gd.pow(2) + bd.pow(2);
                (distance, index)
            })
            .min_by(|(d1, _), (d2, _)| d1.cmp(d2))
            .unwrap()
            .1;

        match index {
            0 => text.black(),
            1 => text.red(),
            2 => text.green(),
            3 => text.yellow(),
            4 => text.blue(),
            5 => text.magenta(),
            6 => text.cyan(),
            7 => text.white(),
            8 => text.bright_black(),
            9 => text.bright_red(),
            10 => text.bright_green(),
            11 => text.bright_yellow(),
            12 => text.bright_blue(),
            13 => text.bright_magenta(),
            14 => text.bright_cyan(),
            15 => text.bright_white(),
            _ => text.white().clear(),
        }
    }

    /// Gets the closest plain color to the TrueColor
    pub fn name_in_closest_terminal_color(&self, red: u8, green: u8, blue: u8) -> ColoredString {
        self.text_in_closest_terminal_color(&self.name, red, green, blue)
    }

    pub fn name_like_project_color(&self, text: &str) -> ColoredString {
        match Rgb::from_hex_str(self.color.as_str()) {
            Ok(color) => {
                let red = color.get_red().round() as u8;
                let green = color.get_green().round() as u8;
                let blue = color.get_blue().round() as u8;

                if HAS_TRUECOLOR_SUPPORT.to_owned() {
                    text.truecolor(red, green, blue).bold()
                } else {
                    self.text_in_closest_terminal_color(text, red, green, blue)
                        .bold()
                }
            }
            Err(_) => text.bold(),
        }
    }
}

impl std::fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match Rgb::from_hex_str(self.color.as_str()) {
                Ok(color) => {
                    let red = color.get_red().round() as u8;
                    let green = color.get_green().round() as u8;
                    let blue = color.get_blue().round() as u8;

                    if HAS_TRUECOLOR_SUPPORT.to_owned() {
                        self.name.truecolor(red, green, blue).bold()
                    } else {
                        self.name_in_closest_terminal_color(red, green, blue).bold()
                    }
                }
                Err(_) => self.name.bold(),
            }
        )
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Client {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Task {
    pub id: i64,
    pub name: String,
    pub workspace_id: i64,
    pub project: Project,
}

impl TimeEntry {
    pub fn get_description(&self) -> String {
        match self.description.as_ref() {
            "" => constants::NO_DESCRIPTION.to_string(),
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
        format!(
            "{}:{:02}:{:02}",
            duration.num_hours(),
            duration.num_minutes() % 60,
            duration.num_seconds() % 60
        )
    }

    pub fn is_running(&self) -> bool {
        self.duration.is_negative()
    }

    pub fn as_running_time_entry(&self, start: DateTime<Utc>) -> TimeEntry {
        TimeEntry {
            start,
            stop: None,
            duration: -start.timestamp(),
            created_with: Some(constants::CLIENT_NAME.to_string()),
            ..self.clone()
        }
    }

    pub fn as_stopped_time_entry(&self, stop: DateTime<Utc>) -> TimeEntry {
        TimeEntry {
            stop: Some(stop),
            duration: (stop - self.start).num_seconds(),
            ..self.clone()
        }
    }

    pub fn get_display_tags(&self) -> String {
        if self.tags.is_empty() {
            "".to_string()
        } else {
            format!("[{}]", self.tags.join(", "))
        }
    }
}

impl Default for TimeEntry {
    fn default() -> Self {
        let start = Utc::now();
        Self {
            id: -1,
            created_with: Some(constants::CLIENT_NAME.to_string()),
            billable: false,
            description: "".to_string(),
            duration: -start.timestamp(),
            project: None,
            start,
            stop: None,
            tags: Vec::new(),
            task: None,
            workspace_id: -1,
        }
    }
}

impl std::fmt::Display for TimeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = format!(
            //{$/space} [{duration}]{running indicator/space} - {description/No Description}{@project/empty string}{(Task: ...)/empty string} {#tags/empty string}
            "{} [{}]{} –  {}{} {}",
            if self.billable {
                "$".green().bold().to_string()
            } else {
                " ".to_string()
            },
            if self.is_running() {
                self.get_duration_hmmss().green().bold()
            } else {
                self.get_duration_hmmss().normal()
            },
            if self.is_running() { "*" } else { " " },
            self.get_description().replace('\n', " "),
            match (self.project.clone(), self.task.clone()) {
                (Some(p), Some(t)) => {
                    format!(
                        " @{}",
                        p.name_like_project_color(&format!("{}: {}", p.name, t.name))
                    )
                }
                (Some(p), None) => format!(" @{p}"),
                (None, Some(t)) => {
                    format!(
                        " @{}",
                        t.project
                            .name_like_project_color(&format!("{}: {}", t.project.name, t.name))
                    )
                }
                (None, None) => "".to_string(),
            },
            if self.tags.is_empty() {
                "".to_string()
            } else {
                format!("#{}", self.get_display_tags().italic())
            }
        );
        write!(f, "{summary}")
    }
}

impl Default for Project {
    fn default() -> Self {
        Self {
            id: constants::DEFAULT_ENTITY_ID,
            name: constants::NO_PROJECT.to_string(),
            workspace_id: constants::DEFAULT_ENTITY_ID,
            client: None,
            is_private: false,
            active: true,
            at: Utc::now(),
            created_at: Utc::now(),
            color: "0".to_string(),
            billable: None,
        }
    }
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: constants::DEFAULT_ENTITY_ID,
            name: constants::NO_TASK.to_string(),
            workspace_id: constants::DEFAULT_ENTITY_ID,
            project: Project::default(),
        }
    }
}

const PARCEL_DESCRIPTION: &str = "Description";
const PARCEL_START: &str = "Start";
const PARCEL_STOP: &str = "Stop";
const PARCEL_BILLABLE: &str = "Billable";
const PARCEL_TAGS: &str = "Tags";
const PARCEL_PROJECT: &str = "Project";
const PARCEL_TASK: &str = "Task";

impl Parcel for TimeEntry {
    fn serialize(&self) -> Vec<u8> {
        let mut out = String::new();
        out.push_str(&format!("{PARCEL_DESCRIPTION}: {}\n", self.description));
        out.push_str(&format!("{PARCEL_START}: {}\n", self.start));
        if let Some(stop) = self.stop {
            out.push_str(&format!("{PARCEL_STOP}: {stop}\n"));
        }
        out.push_str(&format!("{PARCEL_BILLABLE}: {}\n", self.billable));
        out.push_str(&format!("{PARCEL_TAGS}: {}\n", self.tags.join(", ")));
        out.push_str(&format!(
            "{PARCEL_PROJECT}: {}\n",
            self.project.as_ref().map(|p| p.name.as_str()).unwrap_or("")
        ));
        out.push_str(&format!(
            "{PARCEL_TASK}: {}\n",
            self.task.as_ref().map(|t| t.name.as_str()).unwrap_or("")
        ));
        out.into_bytes()
    }

    fn deserialize(data: Vec<u8>, base: &Self) -> ResultWithDefaultError<Self> {
        let text = String::from_utf8(data).map_err(|e| -> Box<dyn std::error::Error + Send> {
            Box::new(std::io::Error::other(format!(
                "edited buffer is not valid UTF-8: {e}"
            )))
        })?;
        let mut entry = base.clone();
        for (line_no, raw) in text.lines().enumerate() {
            let line = raw.trim_end();
            if line.is_empty() {
                continue;
            }
            let Some((key, value)) = line.split_once(':') else {
                continue;
            };
            let key = key.trim();
            let value = value.trim();
            match key {
                PARCEL_DESCRIPTION => entry.description = value.to_string(),
                PARCEL_START => entry.start = parse_field(PARCEL_START, value, line_no)?,
                PARCEL_STOP => {
                    entry.stop = if value.is_empty() {
                        None
                    } else {
                        Some(parse_field(PARCEL_STOP, value, line_no)?)
                    };
                }
                PARCEL_BILLABLE => entry.billable = parse_field(PARCEL_BILLABLE, value, line_no)?,
                PARCEL_TAGS => {
                    entry.tags = if value.is_empty() {
                        Vec::new()
                    } else {
                        value
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    };
                }
                PARCEL_PROJECT => {
                    entry.project = if value.is_empty() {
                        None
                    } else {
                        Some(Project {
                            name: value.to_string(),
                            workspace_id: entry.workspace_id,
                            ..Project::default()
                        })
                    };
                }
                PARCEL_TASK => {
                    entry.task = if value.is_empty() {
                        None
                    } else {
                        Some(Task {
                            name: value.to_string(),
                            workspace_id: entry.workspace_id,
                            project: entry.project.clone().unwrap_or_default(),
                            ..Task::default()
                        })
                    };
                }
                _ => {}
            }
        }
        Ok(entry)
    }
}

fn parse_field<T: std::str::FromStr>(
    field: &str,
    value: &str,
    line_no: usize,
) -> ResultWithDefaultError<T>
where
    T::Err: std::fmt::Display,
{
    value
        .parse::<T>()
        .map_err(|e| -> Box<dyn std::error::Error + Send> {
            Box::new(std::io::Error::other(format!(
                "line {}: invalid {field} \"{value}\": {e}",
                line_no + 1
            )))
        })
}
