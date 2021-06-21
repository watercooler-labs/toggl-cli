use crate::api;
use crate::constants;
use crate::models;
use api::ApiClient;
use chrono::Utc;
use colored::Colorize;
use models::{ResultWithDefaultError, TimeEntry};

pub struct ContinueCommand;

impl ContinueCommand {
    pub async fn execute(api_client: impl ApiClient) -> ResultWithDefaultError<()> {
        let time_entries = api_client.get_time_entries().await?;
        match time_entries.first() {
            None => println!("{}", "No time entries in last 90 days".red()),
            Some(time_entry) => {
                let start = Utc::now();
                let time_entry_to_create = TimeEntry {
                    start,
                    stop: None,
                    duration: -start.timestamp(),
                    created_with: Some(constants::CLIENT_NAME.to_string()),
                    ..time_entry.clone()
                };
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
