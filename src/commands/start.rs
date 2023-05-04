use crate::api;
use crate::commands;
use crate::models;
use crate::models::Entities;
use crate::models::Project;
use crate::picker::ItemPicker;
use crate::picker::PickableItem;
use crate::picker::PickableItemKind;
use crate::utilities;
use api::client::ApiClient;
use colored::Colorize;
use commands::stop::{StopCommand, StopCommandOrigin};
use models::ResultWithDefaultError;
use models::TimeEntry;

pub struct StartCommand;

fn interactively_create_time_entry(
    workspace_id: i64,
    entities: Entities,
    picker: Box<dyn ItemPicker>,
    description: Option<String>,
    project: Option<Project>,
    billable: bool,
) -> TimeEntry {
    let yes_or_default_no = [
        "y".to_string(),
        "n".to_string(),
        "N".to_string(),
        "".to_string(),
    ];

    let description = description.unwrap_or(utilities::read_from_stdin("Description: "));

    let (project, task) = match project {
        Some(_) => (project, None),
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
    let billable = billable
        || project.clone().and_then(|p| p.billable).unwrap_or(
            utilities::read_from_stdin_with_constraints(
                "Is your time entry billable? (y/N): ",
                &yes_or_default_no,
            ) == "y",
        );

    TimeEntry {
        billable,
        description,
        workspace_id,
        project,
        task,
        ..Default::default()
    }
}

impl StartCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        picker: Box<dyn ItemPicker>,
        description: Option<String>,
        project_name: Option<String>,
        billable: bool,
        interactive: bool,
    ) -> ResultWithDefaultError<()> {
        StopCommand::execute(&api_client, StopCommandOrigin::StartCommand).await?;

        let workspace_id = (api_client.get_user().await?).default_workspace_id;
        let entities = api_client.get_entities().await?;

        let project = project_name.and_then(|name| {
            entities
                .projects
                .clone()
                .into_values()
                .find(|p| p.name.to_lowercase() == name.to_lowercase())
        });

        let time_entry_to_create = if interactive {
            interactively_create_time_entry(
                workspace_id,
                entities,
                picker,
                description,
                project,
                billable,
            )
        } else {
            TimeEntry {
                billable: billable || project.clone().and_then(|p| p.billable).unwrap_or(false),
                description: description.unwrap_or("".to_string()),
                project: project.clone(),
                workspace_id,
                ..Default::default()
            }
        };

        let started_entry_id = api_client.create_time_entry(time_entry_to_create).await?;
        let entities = api_client.get_entities().await?;
        let started_entry = entities
            .time_entries
            .iter()
            .find(|te| te.id == started_entry_id)
            .unwrap();

        println!("{}\n{}", "Time entry started".green(), started_entry);

        Ok(())
    }
}
