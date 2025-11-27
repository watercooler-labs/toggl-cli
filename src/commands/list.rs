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
        json_flag: bool,
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
                match entity.unwrap_or(Entity::TimeEntry { json: false }) {
                    Entity::TimeEntry { json: entity_json } => {
                        let json = json_flag || entity_json;
                        let entries = entities
                            .time_entries
                            .iter()
                            .take(count.unwrap_or(usize::MAX))
                            .collect::<Vec<_>>();

                        if json {
                            let json_string = serde_json::to_string_pretty(&entries)
                                .expect("failed to serialize time entries to JSON");
                            writeln!(handle, "{json_string}").expect("failed to print");
                        } else {
                            entries.iter().for_each(|time_entry| {
                                writeln!(handle, "{time_entry}").expect("failed to print")
                            });
                        }
                    }

                    Entity::Project { json: entity_json } => {
                        let json = json_flag || entity_json;
                        let projects = entities
                            .projects
                            .iter()
                            .map(|(_, project)| project)
                            .take(count.unwrap_or(usize::MAX))
                            .collect::<Vec<_>>();

                        if json {
                            let json_string = serde_json::to_string_pretty(&projects)
                                .expect("failed to serialize projects to JSON");
                            writeln!(handle, "{json_string}").expect("failed to print");
                        } else {
                            projects.iter().for_each(|project| {
                                writeln!(handle, "{project}").expect("failed to print")
                            });
                        }
                    }
                };
            }
        }
        Ok(())
    }
}
