use crate::api;
use crate::models;
use api::ApiClient;
use chrono::Utc;
use colored::control;
use colored::Colorize;
use models::{ResultWithDefaultError, TimeEntry};
use skim::prelude::*;

pub struct ContinueCommand;

impl SkimItem for TimeEntry {
    fn text(&self) -> Cow<str> {
        // Wrap text calculation in this block
        // to disable outputing the color keycodes
        // since the filter window doesn't support them
        control::set_override(false);
        let self_as_plainstring = Cow::from(self.to_string());
        control::unset_override();
        self_as_plainstring
    }

    fn output(&self) -> Cow<str> {
        Cow::from(self.id.to_string())
    }
}

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
        get_time_entry_from_user(time_entries)
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
        _ => 1,
    };
    return time_entries.get(continue_entry_index).cloned();
}

fn get_time_entry_from_user(time_entries: Vec<TimeEntry>) -> Option<TimeEntry> {
    let (options, source) = get_skim_configuration(time_entries.clone());
    Skim::run_with(&options, Some(source))
        .map(|output| {
            if output.is_abort {
                println!("{}", "Operation cancelled".red());
                Vec::new()
            } else {
                output.selected_items
            }
        })
        .map(|selected_items| {
            selected_items
                .first()
                .map(|item| item.output().parse::<i64>().unwrap())
        })
        .and_then(|selected_time_entry_id| match selected_time_entry_id {
            Some(time_entry_id) => time_entries
                .iter()
                .find(|time_entry| time_entry.id == time_entry_id)
                .cloned(),
            _ => None,
        })
}

fn get_skim_configuration(
    time_entries: Vec<TimeEntry>,
) -> (SkimOptions<'static>, SkimItemReceiver) {
    let options = SkimOptionsBuilder::default()
        // Set viewport to take entire screen
        .height(Some("100%"))
        // Disable multiselect
        .multi(false)
        .build()
        .unwrap();

    let (sender, source): (SkimItemSender, SkimItemReceiver) = unbounded();
    for time_entry in time_entries {
        // Send time-entries to Skim receiver
        let _ = sender.send(Arc::new(time_entry.clone()));
    }
    // Complete sender transaction to signal no new items will be added after this
    drop(sender);
    (options, source)
}
