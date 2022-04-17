use crate::api;
use crate::commands;
use crate::models;
use api::ApiClient;
use colored::Colorize;
use commands::stop::{StopCommand, StopCommandOrigin};
use models::ResultWithDefaultError;
use models::TimeEntry;

pub struct StartCommand;

impl StartCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        billable: bool,
        description: String,
    ) -> ResultWithDefaultError<()> {
        StopCommand::execute(&api_client, StopCommandOrigin::StartCommand).await?;

        let started_entry = api_client
            .create_time_entry(TimeEntry {
                billable,
                description,
                workspace_id: (api_client.get_user().await?).default_workspace_id,
                ..Default::default()
            })
            .await?;

        println!("{}\n{}", "Time entry started".green(), started_entry);

        Ok(())
    }
}
