use crate::api::client::ApiClient;
use crate::commands::stop::{StopCommand, StopCommandOrigin};
use crate::models::{ResultWithDefaultError, TimeEntry};
use crate::picker::{ItemPicker, PickableItem};
use chrono::Utc;
use colored::Colorize;

pub struct ContinueCommand;

impl ContinueCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        picker: Option<Box<dyn ItemPicker>>,
    ) -> ResultWithDefaultError<()> {
        let entities = api_client.get_entities().await?;
        if entities.time_entries.is_empty() {
            println!("{}", "No time entries in last 90 days".red());
            return Ok(());
        }

        let time_entry_to_continue = match picker {
            None => get_first_stopped_time_entry(entities.time_entries),
            Some(picker) => {
                let items = entities
                    .time_entries
                    .iter()
                    .map(|entry| PickableItem::from_time_entry(entry.clone()))
                    .collect();
                let picked_key = picker.pick(items)?;
                Some(
                    entities
                        .time_entries
                        .iter()
                        .find(|entry| entry.id == picked_key.id)
                        .cloned()
                        .ok_or_else(|| {
                            Box::new(std::io::Error::other(
                                "Selected time entry no longer exists",
                            )) as Box<dyn std::error::Error + Send>
                        })?,
                )
            }
        };

        match time_entry_to_continue {
            None => println!("{}", "No time entry to continue".red()),
            Some(time_entry) => {
                // Resolve a candidate (and allow cancellation) before stopping anything.
                StopCommand::execute(&api_client, StopCommandOrigin::ContinueCommand).await?;
                let mut continued_entry = time_entry.as_running_time_entry(Utc::now());
                continued_entry.id = api_client
                    .create_time_entry(continued_entry.clone())
                    .await?;
                println!(
                    "{}\n{}",
                    "Time entry continued successfully".green(),
                    continued_entry
                );
            }
        }
        Ok(())
    }
}

fn get_first_stopped_time_entry(time_entries: Vec<TimeEntry>) -> Option<TimeEntry> {
    time_entries.into_iter().find(|entry| !entry.is_running())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        api::client::MockApiClient,
        test_support::{self, CancelPicker},
    };

    #[test]
    fn finds_a_stopped_entry_regardless_of_running_entry_position() {
        let running = TimeEntry {
            id: 1,
            ..TimeEntry::default()
        };
        let stopped = TimeEntry {
            id: 2,
            ..TimeEntry::default().as_stopped_time_entry(Utc::now())
        };
        for entries in [
            vec![running.clone(), stopped.clone()],
            vec![stopped.clone(), running.clone()],
        ] {
            assert_eq!(get_first_stopped_time_entry(entries).unwrap().id, 2);
        }
        assert!(get_first_stopped_time_entry(vec![running]).is_none());
        assert!(get_first_stopped_time_entry(Vec::new()).is_none());
    }

    #[tokio::test]
    async fn cancellation_or_no_candidate_does_not_stop_a_timer() {
        for interactive in [false, true] {
            let mut client = MockApiClient::new();
            client.expect_get_entities().times(1).returning(|| {
                let mut entities = test_support::entities();
                entities.time_entries.push(TimeEntry::default());
                Ok(entities)
            });
            let picker = interactive.then(|| Box::new(CancelPicker) as Box<dyn ItemPicker>);
            let result = ContinueCommand::execute(client, picker).await;
            assert_eq!(result.is_err(), interactive);
        }
    }

    #[tokio::test]
    async fn continues_after_stopping_without_a_fragile_refetch() {
        let mut client = MockApiClient::new();
        client.expect_get_entities().times(2).returning(|| {
            let mut entities = test_support::entities();
            entities.time_entries = vec![
                TimeEntry {
                    id: 1,
                    ..TimeEntry::default()
                },
                TimeEntry {
                    id: 2,
                    description: "resume me".into(),
                    duration: 60,
                    stop: Some(Utc::now()),
                    ..TimeEntry::default()
                },
            ];
            Ok(entities)
        });
        let mut sequence = mockall::Sequence::new();
        client
            .expect_update_time_entry()
            .times(1)
            .in_sequence(&mut sequence)
            .withf(|entry| entry.id == 1 && !entry.is_running())
            .returning(|_| Ok(1));
        client
            .expect_create_time_entry()
            .times(1)
            .in_sequence(&mut sequence)
            .withf(|entry| entry.description == "resume me" && entry.is_running())
            .returning(|_| Ok(3));
        ContinueCommand::execute(client, None).await.unwrap();
    }
}
