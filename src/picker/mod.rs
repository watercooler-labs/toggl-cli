mod fzf;
#[cfg(unix)]
mod skim;
use std::collections::HashMap;

use crate::{models, constants::{PROJECT_NOT_FOUND, self}};
use chrono::format;
use models::{ResultWithDefaultError, TimeEntry, Project};

pub struct PickableItem {
    id: i64,
    formatted: String,
}

impl PickableItem {
    pub fn from_time_entry(time_entry: TimeEntry, projects: HashMap<i64, Project>) -> PickableItem {
        let formatted_time_entry = format!(
            "{} {} - {} {}",
            if time_entry.billable { "$" } else { " " },
            time_entry.description,
            match time_entry.project_id {
                // TODO: Print the actual project name here.
                Some(project_id) => match projects.get(&project_id) {
                    Some(project) => project.name.as_str(),
                    None => constants::PROJECT_NOT_FOUND
                }
                None => constants::NO_PROJECT,
            },
            time_entry.get_display_tags()
        );

        PickableItem {
            id: time_entry.id,
            formatted: formatted_time_entry,
        }
    }
}

pub trait ItemPicker {
    fn pick(&self, items: Vec<PickableItem>) -> ResultWithDefaultError<i64>;
}

#[cfg(unix)]
pub fn get_picker(force_fzf: bool) -> Box<dyn ItemPicker> {
    if force_fzf {
        Box::new(fzf::FzfPicker)
    } else {
        Box::new(skim::SkimPicker)
    }
}

#[cfg(not(unix))]
pub fn get_picker(_force_fzf: bool) -> Box<dyn ItemPicker> {
    Box::new(fzf::FzfPicker)
}
