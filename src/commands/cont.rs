use crate::api;
use crate::models;
use api::ApiClient;
use chrono::Utc;
use colored::Colorize;
use models::{ResultWithDefaultError, TimeEntry};

pub struct ContinueCommand;

impl ContinueCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        interactive: bool,
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

        match get_time_entry_to_continue(time_entries, running_time_entry, interactive) {
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

fn get_time_entry_to_continue(
    time_entries: Vec<TimeEntry>,
    running_time_entry: Option<TimeEntry>,
    interactive: bool,
) -> Option<TimeEntry> {
    if interactive {
        autocomplete::get_time_entry_from_user(time_entries)
    } else {
        get_first_stopped_time_entry(time_entries, running_time_entry)
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
