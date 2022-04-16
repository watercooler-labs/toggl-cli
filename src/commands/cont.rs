use crate::api;
use crate::models;
use crate::picker;
use api::ApiClient;
use chrono::Utc;
use colored::Colorize;
use models::{ResultWithDefaultError, TimeEntry};
use picker::{ItemPicker, PickableItem};

pub struct ContinueCommand;

impl ContinueCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        picker: Option<Box<dyn ItemPicker>>,
    ) -> ResultWithDefaultError<()> {
        let running_entry_stop_time = Utc::now();
        let running_time_entry = api_client.get_running_time_entry().await?;
        if let Some(time_entry) = &running_time_entry {
            let stopped_time_entry = api_client
                .update_time_entry(time_entry.as_stopped_time_entry(running_entry_stop_time))
                .await?;

            println!(
                "{}\n{}",
                "Running time entry stopped".yellow(),
                stopped_time_entry
            );
        }

        let time_entries = api_client.get_time_entries().await?;
        if time_entries.is_empty() {
            println!("{}", "No time entries in last 90 days".red());
            return Ok(());
        }

        let time_entry_to_continue = match picker {
            None => get_first_stopped_time_entry(time_entries, running_time_entry),
            Some(time_entry_picker) => {
                let pickable_items = time_entries
                    .iter()
                    .map(|te| PickableItem::from_time_entry(te.clone()))
                    .collect();
                let picked_id = time_entry_picker.pick(pickable_items)?;
                let picked_time_entry = time_entries.iter().find(|te| te.id == picked_id).unwrap();
                Some(picked_time_entry.clone())
            }
        };

        match time_entry_to_continue {
            None => println!("{}", "No time entry to continue".red()),
            Some(time_entry) => {
                let start_time = Utc::now();
                let time_entry_to_create = time_entry.as_running_time_entry(start_time);
                let continued_entry = api_client.create_time_entry(time_entry_to_create).await?;
                println!(
                    "{}\n{}",
                    "Time entry continued successfully".green(),
                    continued_entry
                )
            }
        }

        Ok(())
    }
}

fn get_first_stopped_time_entry(
    time_entries: Vec<TimeEntry>,
    running_time_entry: Option<TimeEntry>,
) -> Option<TimeEntry> {
    // Don't continue a running entry that was just stopped.
    let continue_entry_index = match running_time_entry {
        None => 0,
        Some(_) => 1,
    };
    return time_entries.get(continue_entry_index).cloned();
}
