mod fzf;
#[cfg(unix)]
mod skim;

use std::borrow::BorrowMut;
use std::fmt::Display;
use std::str::FromStr;

use crate::constants;
use crate::models;
use crate::models::Project;
use crate::models::Task;
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

impl FromStr for PickableItemId {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split_string = s.split(' ');
        let pieces = split_string.borrow_mut();
        let kind = match pieces.next().unwrap() {
            "TimeEntry" => PickableItemKind::TimeEntry,
            "Project" => PickableItemKind::Project,
            "Task" => PickableItemKind::Task,
            _ => return Err(()),
        };

        let id = match pieces.next().unwrap().parse::<i64>() {
            Ok(id) => id,
            Err(_) => return Err(()),
        };

        Ok(PickableItemId { kind, id })
    }
}

impl Display for PickableItemId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {}",
            match self.kind {
                PickableItemKind::TimeEntry => "TimeEntry",
                PickableItemKind::Project => "Project",
                PickableItemKind::Task => "Task",
            },
            self.id
        )
    }
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

    pub fn from_project(project: Project) -> PickableItem {
        let formatted_project = format!(
            "{}{}",
            project.name,
            match project.client.clone() {
                Some(c) => format!(" - Client: {}", c.name),
                None => "".to_string(),
            }
        );

        PickableItem {
            id: PickableItemId {
                id: project.id,
                kind: PickableItemKind::Project,
            },
            formatted: formatted_project,
        }
    }

    pub fn from_task(task: Task) -> PickableItem {
        let formatted_task = format!(
            "{} (Task: {}){}",
            task.project.name,
            task.name,
            match task.project.client.clone() {
                Some(c) => format!(" - Client: {}", c.name),
                None => "".to_string(),
            }
        );

        PickableItem {
            id: PickableItemId {
                id: task.id,
                kind: PickableItemKind::Task,
            },
            formatted: formatted_task,
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
