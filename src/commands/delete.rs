use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct DeleteCommand;

impl DeleteCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        id: i64,
    ) -> ResultWithDefaultError<()> {
        let entities = api_client.get_entities().await?;
        let time_entry = entities.time_entries.iter().find(|te| te.id == id).cloned();

        match time_entry {
            None => println!("{}", format!("No time entry found with id {id}").yellow()),
            Some(entry) => match api_client.delete_time_entry(entry.workspace_id, id).await {
                Err(error) => println!("{}\n{}", "Couldn't delete time entry".red(), error),
                Ok(()) => println!("{}\n{}", "Time entry deleted successfully".green(), entry),
            },
        }

        Ok(())
    }
}
