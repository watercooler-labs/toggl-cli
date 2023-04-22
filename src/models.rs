use std::{cmp, env};

use crate::constants;
use chrono::{DateTime, Duration, Utc};
use colored::{ColoredString, Colorize};
use colors_transform::{Color, Rgb};
use lazy_static::lazy_static;
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
pub struct Project {
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
    // "billable":null,
    // "template":null,
    // "auto_estimates":null,
    // "estimated_hours":null,
    // "rate":null,
    // "rate_last_updated":null,
    // "currency":null,
    // "recurring":false,
    // "recurring_parameters":null,
    // "current_period":null,
    // "fixed_fee":null,
    // "actual_hours":142,
}

lazy_static! {
    pub static ref HAS_TRUECOLOR_SUPPORT: bool = if let Ok(truecolor) = env::var("COLORTERM") {
        truecolor == "truecolor" || truecolor == "24bit"
    } else {
        false
    };
}

impl Project {
    /// Gets the closest plain color to the TrueColor
    pub fn name_in_closest_terminal_color(&self, red: u8, green: u8, blue: u8) -> ColoredString {
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
            0 => self.name.black(),
            1 => self.name.red(),
            2 => self.name.green(),
            3 => self.name.yellow(),
            4 => self.name.blue(),
            5 => self.name.magenta(),
            6 => self.name.cyan(),
            7 => self.name.white(),
            8 => self.name.bright_black(),
            9 => self.name.bright_red(),
            10 => self.name.bright_green(),
            11 => self.name.bright_yellow(),
            12 => self.name.bright_blue(),
            13 => self.name.bright_magenta(),
            14 => self.name.bright_cyan(),
            15 => self.name.bright_white(),
            _ => self.name.white().clear(),
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

#[derive(Serialize, Deserialize, Clone)]
pub struct TimeEntry {
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
            project_id: None,
            start,
            stop: None,
            tags: Vec::new(),
            task_id: None,
            workspace_id: -1,
        }
    }
}

impl std::fmt::Display for TimeEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = format!(
            "[{}]{} –  {} {}",
            if self.is_running() {
                self.get_duration_hmmss().green().bold()
            } else {
                self.get_duration_hmmss().normal()
            },
            if self.is_running() { "*" } else { " " },
            self.get_description(),
            self.get_display_tags().italic()
        );
        write!(f, "{}", summary)
    }
}
