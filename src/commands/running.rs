use crate::api;
use crate::models;
use api::client::ApiClient;
use colored::Colorize;
use models::ResultWithDefaultError;

pub struct RunningTimeEntryCommand;

impl RunningTimeEntryCommand {
    pub async fn execute(api_client: impl ApiClient) -> ResultWithDefaultError<()> {
        let entities = api_client.get_entities().await?;
        match entities.running_time_entry() {
            None => println!("{}", "No time entry is running at the moment".yellow()),
            Some(running_time_entry) => println!("{}", running_time_entry),
        }

        Ok(())
    }
}
