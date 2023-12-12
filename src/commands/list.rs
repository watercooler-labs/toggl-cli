use crate::api;
use crate::arguments::Entity;
use crate::models;
use api::client::ApiClient;
use colored::Colorize;
use models::ResultWithDefaultError;

pub struct ListCommand;

impl ListCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        count: Option<usize>,
        entity: Option<Entity>,
    ) -> ResultWithDefaultError<()> {
        match api_client.get_entities().await {
            Err(error) => println!(
                "{}\n{}",
                "Couldn't fetch time entries the from API".red(),
                error
            ),
            Ok(entities) => match entity.unwrap_or(Entity::TimeEntry) {
                Entity::TimeEntry => entities
                    .time_entries
                    .iter()
                    .take(count.unwrap_or(usize::max_value()))
                    .for_each(|time_entry| println!("{}", time_entry)),

                Entity::Project => entities
                    .projects
                    .iter()
                    .take(count.unwrap_or(usize::max_value()))
                    .for_each(|(_, projects)| println!("{}", projects)),
            },
        }
        Ok(())
    }
}
