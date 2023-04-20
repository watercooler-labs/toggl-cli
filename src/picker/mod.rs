mod fzf;
#[cfg(unix)]
mod skim;
use crate::models;
use models::{ResultWithDefaultError, TimeEntry};

pub struct PickableItem {
    id: i64,
    formatted: String,
}

impl PickableItem {
    pub fn from_time_entry(time_entry: TimeEntry) -> PickableItem {
        let formatted_time_entry = format!(
            "{} {} - {}{}",
            if time_entry.billable { "$" } else { " " },
            time_entry.description,
            match time_entry.project_id {
                // TODO: Print the actual project name here.
                Some(_) => "With project",
                None => "No project",
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
