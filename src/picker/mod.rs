mod fzf;
#[cfg(unix)]
mod skim;
use crate::models;
use models::{ResultWithDefaultError, TimeEntry};

pub trait PickableItem: Clone {
    fn id(&self) -> i64;
    fn formatted(&self) -> String;
}

impl PickableItem for TimeEntry {
    fn id(&self) -> i64 {
        self.id
    }

    fn formatted(&self) -> String {
        format!(
            "{} {} - {}",
            if self.billable { "$" } else { " " },
            self.description,
            match self.project_id {
                // TODO: Print the actual project name here.
                Some(_) => "With project",
                None => "No project",
            },
            // TODO: Display tags
        )
    }
}

pub trait ItemPicker {
    fn pick<T: PickableItem>(&self, items: Vec<T>) -> ResultWithDefaultError<T>;
}

#[cfg(unix)]
pub fn get_picker(force_fzf: bool) -> impl ItemPicker {
    if force_fzf {
        fzf::FzfPicker
    } else {
        skim::SkimPicker
    }
}

#[cfg(not(unix))]
pub fn get_picker(_force_fzf: bool) -> impl ItemPicker {
    fzf::FzfPicker
}
