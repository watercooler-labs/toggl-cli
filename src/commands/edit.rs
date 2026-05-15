use crate::api::client::ApiClient;
use crate::constants::DEFAULT_ENTITY_ID;
use crate::models::{Entities, ResultWithDefaultError, TimeEntry};
use crate::parcel::Parcel;
use colored::Colorize;

pub struct EditCommand;

impl EditCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        id: Option<i64>,
        description: Option<String>,
        project_name: Option<String>,
        tags: Option<Vec<String>>,
    ) -> ResultWithDefaultError<()> {
        let entities = api_client.get_entities().await?;
        let entry = match select_entry(&entities, id) {
            Some(entry) => entry,
            None => {
                println!("{}", "No matching time entry found".yellow());
                return Ok(());
            }
        };

        let has_flag_edits = description.is_some() || project_name.is_some() || tags.is_some();

        let updated = if has_flag_edits {
            apply_flag_edits(&entities, entry, description, project_name, tags)?
        } else {
            apply_editor_edits(&entities, entry)?
        };

        match api_client.update_time_entry(updated.clone()).await {
            Err(error) => println!("{}\n{}", "Couldn't update time entry".red(), error),
            Ok(_) => println!("{}\n{}", "Time entry updated successfully".green(), updated),
        }
        Ok(())
    }
}

fn select_entry(entities: &Entities, id: Option<i64>) -> Option<TimeEntry> {
    if let Some(id) = id {
        return entities.time_entries.iter().find(|te| te.id == id).cloned();
    }
    entities.running_time_entry().or_else(|| {
        entities
            .time_entries
            .iter()
            .max_by_key(|te| te.start)
            .cloned()
    })
}

fn apply_flag_edits(
    entities: &Entities,
    entry: TimeEntry,
    description: Option<String>,
    project_name: Option<String>,
    tags: Option<Vec<String>>,
) -> ResultWithDefaultError<TimeEntry> {
    let project = match project_name.as_deref() {
        Some("") => None,
        Some(name) => match entities.projects.values().find(|p| p.name == name).cloned() {
            Some(project) => Some(project),
            None => {
                return Err(Box::new(std::io::Error::other(format!(
                    "Project \"{name}\" not found"
                ))));
            }
        },
        None => entry.project.clone(),
    };

    let tags = match tags {
        Some(ref t) if t.len() == 1 && t[0].is_empty() => Vec::new(),
        Some(t) => t,
        None => entry.tags.clone(),
    };

    Ok(TimeEntry {
        description: description.unwrap_or(entry.description.clone()),
        project,
        tags,
        ..entry
    })
}

fn apply_editor_edits(entities: &Entities, entry: TimeEntry) -> ResultWithDefaultError<TimeEntry> {
    let edited = entry.update_in_editor()?;

    let project = match edited.project {
        Some(p) if p.id != DEFAULT_ENTITY_ID => Some(p),
        Some(p) => Some(
            entities
                .project_for_name(edited.workspace_id, &p.name)
                .ok_or_else(|| -> Box<dyn std::error::Error + Send> {
                    Box::new(std::io::Error::other(format!(
                        "Project \"{}\" not found",
                        p.name
                    )))
                })?,
        ),
        None => None,
    };

    let task = match edited.task {
        Some(t) if t.id != DEFAULT_ENTITY_ID => Some(t),
        Some(t) => {
            let project_id = project.as_ref().map(|p| p.id).ok_or_else(
                || -> Box<dyn std::error::Error + Send> {
                    Box::new(std::io::Error::other(format!(
                        "Task \"{}\" requires a project",
                        t.name
                    )))
                },
            )?;
            Some(
                entities
                    .task_for_name(edited.workspace_id, project_id, &t.name)
                    .ok_or_else(|| -> Box<dyn std::error::Error + Send> {
                        Box::new(std::io::Error::other(format!(
                            "Task \"{}\" not found in the resolved project",
                            t.name
                        )))
                    })?,
            )
        }
        None => None,
    };

    Ok(TimeEntry {
        project,
        task,
        ..edited
    })
}
