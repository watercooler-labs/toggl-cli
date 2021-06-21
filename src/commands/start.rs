use crate::api;
use crate::models;
use api::ApiClient;
use chrono::Utc;
use colored::Colorize;
use models::ResultWithDefaultError;
use models::TimeEntry;

pub struct StartCommand;

impl StartCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        billable: bool,
        description: String,
    ) -> ResultWithDefaultError<()> {
        let running_entry_stop_time = Utc::now();
        match api_client.get_running_time_entry().await {
            Err(error) => {
                println!(
                    "{} {:?}",
                    "Couldn't retrieve running time entry.".red(),
                    error
                );
                return Ok(());
            }
            Ok(value) => {
                if let Some(time_entry) = value {
                    if let Err(stop_error) = api_client
                        .update_time_entry(
                            time_entry.as_stopped_time_entry(running_entry_stop_time),
                        )
                        .await
                    {
                        println!(
                            "{} {:?}",
                            "Couldn't stop running time entry.".red(),
                            stop_error
                        );
                        return Ok(());
                    }
                    println!("Running time entry stopped")
                }
            }
        }

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
