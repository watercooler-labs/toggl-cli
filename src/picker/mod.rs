mod fzf;
#[cfg(unix)]
mod skim;

use crate::constants;
use crate::models;

use models::{Project, ResultWithDefaultError, TimeEntry};

pub struct PickableItem {
    id: i64,
    formatted: String,
}

impl PickableItem {
    pub fn from_time_entry(time_entry: TimeEntry, project: Option<Project>) -> PickableItem {
        let formatted_time_entry = format!(
            "{} {} - {} {}",
            if time_entry.billable { "$" } else { " " },
            time_entry.get_description(),
            match project {
                Some(p) => p.name,
                None => constants::NO_PROJECT.to_string(),
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
