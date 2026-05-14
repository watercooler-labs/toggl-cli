use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct EditCommand;

impl EditCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        id: Option<i64>,
        description: Option<String>,
        project_name: Option<String>,
        tags: Option<Vec<String>>,
    ) -> ResultWithDefaultError<()> {
        let entities = api_client.get_entities().await?;

        let time_entry = match id {
            Some(id) => entities.time_entries.into_iter().find(|te| te.id == id),
            None => entities.running_time_entry(),
        };

        match time_entry {
            None => println!("{}", "No matching time entry found".yellow()),
            Some(entry) => {
                let project = match project_name.as_deref() {
                    Some("") => None,
                    Some(name) => match entities.projects.into_values().find(|p| p.name == name) {
                        Some(project) => Some(project),
                        None => {
                            return Err(Box::new(std::io::Error::other(format!(
                                "Project \"{}\" not found",
                                name
                            ))));
                        }
                    },
                    None => entry.project.clone(),
                };

                let tags = match tags {
                    Some(ref t) if t.len() == 1 && t[0].is_empty() => Vec::new(),
                    Some(t) => t,
                    None => entry.tags.clone(),
                };

                let updated = crate::models::TimeEntry {
                    description: description.unwrap_or(entry.description.clone()),
                    project,
                    tags,
                    ..entry
                };

                match api_client.update_time_entry(updated.clone()).await {
                    Err(error) => println!("{}\n{}", "Couldn't update time entry".red(), error),
                    Ok(_) => println!("{}\n{}", "Time entry updated successfully".green(), updated),
                }
            }
        }

        Ok(())
    }
}
