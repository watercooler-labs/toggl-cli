use crate::api;
use crate::arguments::Entity;
use crate::models;
use api::client::ApiClient;
use colored::Colorize;
use models::ResultWithDefaultError;
use std::io::{self, BufWriter, Write};

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
            Ok(entities) => {
                // use this to avoid calling println! in a loop:
                // <https://rust-cli.github.io/book/tutorial/output.html#a-note-on-printing-performance>
                let stdout = io::stdout();
                let mut handle = BufWriter::new(stdout);

                // TODO: better error handling for writeln!
                match entity.unwrap_or(Entity::TimeEntry) {
                    Entity::TimeEntry => entities
                        .time_entries
                        .iter()
                        .take(count.unwrap_or(usize::MAX))
                        .for_each(|time_entry| {
                            writeln!(handle, "{}", time_entry).expect("failed to print")
                        }),

                    Entity::Project => entities
                        .projects
                        .iter()
                        .take(count.unwrap_or(usize::MAX))
                        .for_each(|(_, projects)| {
                            writeln!(handle, "{}", projects).expect("failed to print")
                        }),
                };
            }
        }
        Ok(())
    }
}
