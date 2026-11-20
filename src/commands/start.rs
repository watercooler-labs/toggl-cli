use crate::api;
use crate::commands;
use crate::config;
use crate::models;
use crate::models::Entities;
use crate::picker::ItemPicker;
use crate::picker::PickableItem;
use crate::picker::PickableItemKind;
use crate::utilities;
use api::client::ApiClient;
use colored::Colorize;
use commands::stop::{StopCommand, StopCommandOrigin};
use models::{ResultWithDefaultError, TimeEntry};

pub struct StartCommand;

fn interactively_create_time_entry(
    time_entry: TimeEntry,
    entities: Entities,
    picker: Box<dyn ItemPicker>,
) -> TimeEntry {
    let yes_or_default_no = [
        "y".to_string(),
        "n".to_string(),
        "N".to_string(),
        "".to_string(),
    ];

    let (project, task) = match time_entry.project {
        Some(_) => (time_entry.project, None),
        None => {
            if entities.projects.is_empty() {
                (None, None)
            } else {
                let mut pickable_items: Vec<PickableItem> = entities
                    .projects
                    .clone()
                    .into_values()
                    .map(PickableItem::from_project)
                    .collect();

                pickable_items.extend(
                    entities
                        .tasks
                        .clone()
                        .into_values()
                        .map(PickableItem::from_task),
                );

                match picker.pick(pickable_items) {
                    Ok(picked_key) => match picked_key.kind {
                        PickableItemKind::TimeEntry => (None, None),
                        PickableItemKind::Project => {
                            (entities.projects.get(&picked_key.id).cloned(), None)
                        }
                        PickableItemKind::Task => {
                            let task = entities.tasks.get(&picked_key.id).cloned().unwrap();
                            (Some(task.clone().project), Some(task))
                        }
                    },

                    Err(_) => (None, None),
                }
            }
        }
    };

    // Only ask for billable if the user didn't provide a value AND if the selected project doesn't have a default billable setting.
    let billable = time_entry.billable
        || project.clone().and_then(|p| p.billable).unwrap_or(
            utilities::read_from_stdin_with_constraints(
                "Is your time entry billable? (y/N): ",
                &yes_or_default_no,
            ) == "y",
        );

    let task = task.or(time_entry.task.clone());

    TimeEntry {
        billable,
        project,
        task,
        ..time_entry
    }
}

impl StartCommand {
    #[allow(clippy::too_many_arguments)]
    pub async fn execute(
        api_client: impl ApiClient,
        picker: Box<dyn ItemPicker>,
        description: Option<String>,
        project_name: Option<String>,
        tags: Option<Vec<String>>,
        billable: bool,
        interactive: bool,
        task: Option<String>,
    ) -> ResultWithDefaultError<()> {
        StopCommand::execute(&api_client, StopCommandOrigin::StartCommand).await?;

        let workspace_id = (api_client.get_user().await?).default_workspace_id;
        let entities = api_client.get_entities().await?;

        let default_time_entry = config::locate::locate_config_path()
            .and_then(config::parser::get_config_from_file)
            .and_then(|track_config| track_config.get_default_entry(entities.clone()))
            .unwrap_or_else(|_| TimeEntry::default());

        let workspace_id = if default_time_entry.workspace_id != -1 {
            default_time_entry.workspace_id
        } else {
            workspace_id
        };

        // Look up project by name if provided
        let project_from_flag = project_name.as_ref().and_then(|name| {
            entities
                .projects
                .clone()
                .into_values()
                .find(|p| p.name == *name)
        });

        // Error if project name was provided but not found
        if let Some(name) = &project_name {
            if project_from_flag.is_none() {
                return Err(Box::new(std::io::Error::other(format!(
                    "Project \"{}\" not found",
                    name
                ))));
            }
        }

        // Task names are only unique within a project, so a task is always
        // looked up in the project given alongside it.
        let task_obj = match (&task, &project_from_flag) {
            (None, _) => None,
            (Some(_), None) => {
                return Err(Box::new(std::io::Error::other("--task requires --project")));
            }
            (Some(task_name), Some(project)) => {
                let tasks_in_project = entities
                    .tasks
                    .values()
                    .filter(|t| t.project.id == project.id);

                match tasks_in_project.clone().find(|t| t.name == *task_name) {
                    Some(task) => Some(task.clone()),
                    // A misspelt task is the common case, so name the
                    // candidates instead of leaving the caller to go looking.
                    None => {
                        let mut available =
                            tasks_in_project.map(|t| t.name.clone()).collect::<Vec<_>>();
                        available.sort();

                        let hint = if available.is_empty() {
                            format!("Project \"{}\" has no tasks", project.name)
                        } else {
                            format!("Tasks in \"{}\": {}", project.name, available.join(", "))
                        };

                        return Err(Box::new(std::io::Error::other(format!(
                            "Task \"{}\" not found\n{}",
                            task_name, hint
                        ))));
                    }
                }
            }
        };

        // The config's project and task form a pair. A project from the
        // command line replaces it, keeping the config task only if it
        // belongs to that project.
        let (project, task_obj) = match project_from_flag {
            Some(project) => {
                let task = task_obj.or_else(|| {
                    default_time_entry
                        .task
                        .clone()
                        .filter(|t| t.project.id == project.id)
                });
                (Some(project), task)
            }
            None => (
                default_time_entry.project.clone(),
                default_time_entry.task.clone(),
            ),
        };

        let tags = tags.unwrap_or(default_time_entry.tags.clone());

        let billable = billable
            || default_time_entry.billable
            || project.clone().and_then(|p| p.billable).unwrap_or(false);

        let description = description.unwrap_or(default_time_entry.description.clone());

        let time_entry_to_create = {
            let initial_entry = TimeEntry {
                description,
                project,
                tags,
                billable,
                workspace_id,
                task: task_obj,
                ..TimeEntry::default()
            };
            if interactive {
                interactively_create_time_entry(initial_entry, entities.clone(), picker)
            } else {
                initial_entry
            }
        };

        let started_entry_id = api_client
            .create_time_entry(time_entry_to_create.clone())
            .await;
        if started_entry_id.is_err() {
            println!("{}", "Failed to start time entry".red());
            return Err(started_entry_id.err().unwrap());
        }

        println!("{}\n{}", "Time entry started".green(), time_entry_to_create);

        Ok(())
    }
}
