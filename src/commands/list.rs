use crate::api;
use crate::arguments::EntryType;
use crate::models::{self, Project};
use api::client::ApiClient;
use colored::Colorize;
use models::ResultWithDefaultError;

pub struct ListCommand;

impl ListCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        count: Option<usize>,
        entry_type: Option<EntryType>,
    ) -> ResultWithDefaultError<()> {
        match api_client.get_entities().await {
            Err(error) => println!(
                "{}\n{}",
                "Couldn't fetch time entries the from API".red(),
                error
            ),
            Ok(entities) => match entry_type.unwrap_or(EntryType::Entry) {
                EntryType::Entry => entities
                    .time_entries
                    .iter()
                    .take(count.unwrap_or(usize::max_value()))
                    .for_each(|time_entry| println!("{}", time_entry)),

                EntryType::Project => entities
                    .projects
                    .values()
                    .collect::<Vec<&Project>>()
                    .iter()
                    .take(count.unwrap_or(usize::max_value()))
                    .for_each(|projects| println!("{}", projects)),
            },
        }
        Ok(())
    }
}
