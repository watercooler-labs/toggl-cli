use crate::api;
use crate::models;
use api::ApiClient;
use chrono::Utc;
use colored::Colorize;
use models::ResultWithDefaultError;

pub struct ContinueCommand;

impl ContinueCommand {
    pub async fn execute(api_client: impl ApiClient) -> ResultWithDefaultError<()> {
        let running_entry_stop_time = Utc::now();
        let running_time_entry = api_client.get_running_time_entry().await;
        match &running_time_entry {
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

        let time_entries = api_client.get_time_entries().await?;
        // Don't continue a running entry that was just stopped.
        let continue_entry_index = if let Ok(None) = running_time_entry {
            0
        } else {
            1
        };

        match time_entries.get(continue_entry_index) {
            None => println!("{}", "No time entries in last 90 days".red()),
            Some(time_entry) => {
                let start_time = Utc::now();
                let time_entry_to_create = time_entry.as_running_time_entry(start_time);
                let continued_entry = api_client.create_time_entry(time_entry_to_create).await?;
                println!(
                    "{}\n{}",
                    "Time entry continued successfully".green(),
                    continued_entry
                )
            }
        }

        Ok(())
    }
}
