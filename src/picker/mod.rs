mod fzf;
#[cfg(unix)]
mod skim;

use crate::constants;
use crate::models;

use models::{ResultWithDefaultError, TimeEntry};

#[derive(Clone)]
pub struct PickableItemId {
    pub id: i64,
    pub kind: PickableItemKind,
}

#[derive(Clone, Copy)]
pub enum PickableItemKind {
    TimeEntry,
    Project,
    Task,
}

pub struct PickableItem {
    id: PickableItemId,
    formatted: String,
}

impl PickableItem {
    pub fn from_time_entry(time_entry: TimeEntry) -> PickableItem {
        let formatted_time_entry = format!(
            "{} {} - {} {}",
            if time_entry.billable { "$" } else { " " },
            time_entry.get_description(),
            match time_entry.project.clone() {
                Some(p) => p.name,
                None => constants::NO_PROJECT.to_string(),
            },
            time_entry.get_display_tags()
        );

        PickableItem {
            id: PickableItemId {
                id: time_entry.id,
                kind: PickableItemKind::TimeEntry,
            },
            formatted: formatted_time_entry,
        }
    }
}

pub trait ItemPicker {
    fn pick(&self, items: Vec<PickableItem>) -> ResultWithDefaultError<PickableItemId>;
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
