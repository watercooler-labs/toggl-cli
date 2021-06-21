use crate::api;
use crate::models;
use api::ApiClient;
use chrono::Utc;
use colored::Colorize;
use models::ResultWithDefaultError;

pub struct StopCommand;

impl StopCommand {
    pub async fn execute(api_client: impl ApiClient) -> ResultWithDefaultError<()> {
        match api_client.get_running_time_entry().await? {
            None => println!("{}", "No time entry is running at the moment".yellow()),
            Some(running_time_entry) => {
                let stop_time = Utc::now();
                let stopped_time_entry = running_time_entry.as_stopped_time_entry(stop_time);
                let _ = api_client.update_time_entry(stopped_time_entry).await?;
                println!("{}", "Time entry stopped successfully".green());
            }
        }

        Ok(())
    }
}
