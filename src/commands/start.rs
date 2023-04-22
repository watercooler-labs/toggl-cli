use crate::api;
use crate::commands;
use crate::models;
use crate::utilities;
use api::client::ApiClient;
use colored::Colorize;
use commands::stop::{StopCommand, StopCommandOrigin};
use models::ResultWithDefaultError;
use models::TimeEntry;

pub struct StartCommand;

fn interactively_create_time_entry(workspace_id: i64) -> TimeEntry {
    let yes_or_no = [
        "y".to_string(),
        "n".to_string(),
        "N".to_string(),
        "".to_string(),
    ];
    let description = utilities::read_from_stdin("Description: ");
    let billable = utilities::read_from_stdin_with_constraints(
        "Is your time entry billable? (y/N): ",
        &yes_or_no,
    ) == "y";

    TimeEntry {
        billable,
        description,
        workspace_id,
        ..Default::default()
    }
}

impl StartCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        possible_description: Option<String>,
        billable: bool,
    ) -> ResultWithDefaultError<()> {
        StopCommand::execute(&api_client, StopCommandOrigin::StartCommand).await?;

        let workspace_id = (api_client.get_user().await?).default_workspace_id;
        let time_entry_to_create = match possible_description {
            None => interactively_create_time_entry(workspace_id),
            Some(description) => TimeEntry {
                billable,
                description,
                workspace_id,
                ..Default::default()
            },
        };

        let started_entry_id = api_client.create_time_entry(time_entry_to_create).await?;
        let entities = api_client.get_entities().await?;
        let started_entry = entities.time_entries.get(&started_entry_id).unwrap();

        println!("{}\n{}", "Time entry started".green(), started_entry);

        Ok(())
    }
}
