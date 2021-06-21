use crate::api;
use crate::models;
use api::ApiClient;
use colored::Colorize;
use models::ResultWithDefaultError;

pub struct RunningTimeEntryCommand;

impl RunningTimeEntryCommand {
    pub async fn execute(api_client: impl ApiClient) -> ResultWithDefaultError<()> {
        match api_client.get_running_time_entry().await? {
            None => println!("{}", "No time entry is running at the moment".yellow()),
            Some(running_time_entry) => println!("{}", running_time_entry),
        }

        Ok(())
    }
}
