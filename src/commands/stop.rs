use crate::models;
use crate::api;
use api::ApiClient;
use colored::Colorize;
use models::{ResultWithDefaultError, TimeEntry};
use chrono::Utc;

pub struct StopCommand;

impl StopCommand {
    pub async fn execute(api_client: impl ApiClient) -> ResultWithDefaultError<()> {
        match api_client.get_running_time_entry().await? {
            None => println!("{}", "No time entry is running at the moment".yellow()),
            Some(running_time_entry) => {
                let stop_time = Utc::now();
                let stopped_time_entry = TimeEntry {
                    stop: Some(stop_time),
                    duration: (stop_time - running_time_entry.start).num_seconds(),
                    ..running_time_entry
                };
                let _ = api_client.update_time_entry(stopped_time_entry).await?;
                println!("{}", "Time entry stopped successfully".green());
            }
        }
    
        Ok(())
    }
}