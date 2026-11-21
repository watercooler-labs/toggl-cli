use crate::api::client::ApiClient;
use crate::arguments::Entity;
use crate::models::ResultWithDefaultError;
use serde::Serialize;
use std::io::{self, BufWriter, Write};

pub struct ListCommand;

fn write_items<T: Serialize + std::fmt::Display>(
    writer: &mut impl Write,
    items: &[T],
    json: bool,
) -> ResultWithDefaultError<()> {
    if json {
        write_json(writer, items)?;
    } else {
        for item in items {
            writeln!(writer, "{item}")
                .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send>)?;
        }
    }
    Ok(())
}

fn write_json(
    writer: &mut impl Write,
    value: &(impl Serialize + ?Sized),
) -> ResultWithDefaultError<()> {
    serde_json::to_writer_pretty(&mut *writer, value)
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send>)?;
    writeln!(writer).map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send>)?;
    Ok(())
}

impl ListCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        count: Option<usize>,
        json_flag: bool,
        since: Option<String>,
        until: Option<String>,
        entity: Option<Entity>,
    ) -> ResultWithDefaultError<()> {
        let mut writer = BufWriter::new(io::stdout());
        Self::execute_to(
            api_client,
            count,
            json_flag,
            since,
            until,
            entity,
            &mut writer,
        )
        .await?;
        writer
            .flush()
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send>)?;
        Ok(())
    }

    async fn execute_to(
        api_client: impl ApiClient,
        count: Option<usize>,
        json_flag: bool,
        since: Option<String>,
        until: Option<String>,
        entity: Option<Entity>,
        writer: &mut impl Write,
    ) -> ResultWithDefaultError<()> {
        let limit = count.unwrap_or(usize::MAX);
        if let Some(Entity::Tag { json }) = entity {
            let user = api_client.get_user().await?;
            let tags = api_client.get_tags(user.default_workspace_id).await?;
            let tags = tags.iter().take(limit).collect::<Vec<_>>();
            return write_items(writer, &tags, json_flag || json);
        }

        let entity = entity.unwrap_or(Entity::TimeEntry { json: false });
        if let Entity::TimeEntry { json } = entity {
            if since.is_some() || until.is_some() {
                let entries = api_client.get_time_entries_filtered(since, until).await?;
                let entries = entries.iter().take(limit).collect::<Vec<_>>();
                return write_items(writer, &entries, json_flag || json);
            }
        }

        // Fetch successfully before printing anything: an outage is not an empty list.
        let entities = api_client.get_entities().await?;
        match entity {
            Entity::TimeEntry { json } => {
                let entries = entities.time_entries.iter().take(limit).collect::<Vec<_>>();
                write_items(writer, &entries, json_flag || json)
            }
            Entity::Project { json } => {
                let projects = entities.projects.values().take(limit).collect::<Vec<_>>();
                write_items(writer, &projects, json_flag || json)
            }
            Entity::Task { json, project } => {
                let mut tasks = entities
                    .tasks
                    .values()
                    .filter(|task| {
                        project
                            .as_ref()
                            .is_none_or(|name| task.project.name == *name)
                    })
                    .collect::<Vec<_>>();
                tasks.sort_by(|a, b| {
                    (&a.name, &a.project.name, a.id).cmp(&(&b.name, &b.project.name, b.id))
                });
                tasks.truncate(limit);
                if json_flag || json {
                    write_json(writer, &tasks)
                } else {
                    for task in tasks {
                        if project.is_some() {
                            writeln!(writer, "{}", task.name)
                        } else {
                            writeln!(writer, "{} @{}", task.name, task.project.name)
                        }
                        .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send>)?;
                    }
                    Ok(())
                }
            }
            Entity::Tag { .. } | Entity::Profile { .. } => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{api::client::MockApiClient, error::ApiError};

    #[tokio::test]
    async fn failed_fetches_never_emit_empty_json_or_errors_on_stdout() {
        for since in [None, Some("2026-11-20".to_string())] {
            let mut client = MockApiClient::new();
            if since.is_some() {
                client
                    .expect_get_time_entries_filtered()
                    .returning(|_, _| Err(Box::new(ApiError::Network)));
            } else {
                client
                    .expect_get_entities()
                    .returning(|| Err(Box::new(ApiError::Network)));
            }
            let mut output = Vec::new();
            assert!(
                ListCommand::execute_to(client, None, true, since, None, None, &mut output)
                    .await
                    .is_err()
            );
            assert!(output.is_empty());
        }
    }
}
