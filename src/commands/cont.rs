use crate::api;
use crate::models;
use api::ApiClient;
use chrono::Utc;
use colored::Colorize;
use models::ResultWithDefaultError;

pub struct ContinueCommand;

impl ContinueCommand {
    pub async fn execute(api_client: impl ApiClient) -> ResultWithDefaultError<()> {
        let time_entries = api_client.get_time_entries().await?;
        match time_entries.first() {
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
