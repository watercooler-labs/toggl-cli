use crate::api;
use crate::commands;
use crate::models;
use crate::utilities;
use api::ApiClient;
use colored::Colorize;
use commands::stop::{StopCommand, StopCommandOrigin};
use models::ResultWithDefaultError;
use models::TimeEntry;
use std::io::{self, Write};

pub struct StartCommand;

fn print_without_buffer(text: &str) {
    print!("{}", text);
    io::stdout().flush().unwrap();
}

fn read_from_stdin(text: &str) -> String {
    print_without_buffer(text);
    let mut result = String::new();
    io::stdin()
        .read_line(&mut result)
        .expect("Failed to read line");
    utilities::remove_trailing_newline(result)
}

fn read_from_stdin_with_constraints(text: &str, valid_values: &[String]) -> String {
    loop {
        let result = read_from_stdin(text);
        if valid_values.contains(&result) {
            return result;
        } else {
            let error_message = format!(
                "Invalid value \"{}\". Valid values are: {}\n",
                result,
                valid_values.join(", ")
            )
            .red();
            print_without_buffer(&error_message);
        }
    }
}

fn interactively_create_time_entry(workspace_id: i64) -> TimeEntry {
    let yes_or_no = [
        "y".to_string(),
        "n".to_string(),
        "N".to_string(),
        "".to_string(),
    ];
    let description = read_from_stdin("Description: ");
    let billable =
        read_from_stdin_with_constraints("Is your time entry billable? (y/N): ", &yes_or_no) == "y";

    TimeEntry {
        billable,
        description,
        workspace_id,
        ..Default::default()
    }
}

impl StartCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        possible_description: Option<String>,
        billable: bool,
    ) -> ResultWithDefaultError<()> {
        StopCommand::execute(&api_client, StopCommandOrigin::StartCommand).await?;

        let workspace_id = (api_client.get_user().await?).default_workspace_id;
        let time_entry_to_create = match possible_description {
            None => interactively_create_time_entry(workspace_id),
            Some(description) => TimeEntry {
                billable,
                description,
                workspace_id,
                ..Default::default()
            },
        };

        let started_entry = api_client.create_time_entry(time_entry_to_create).await?;

        println!("{}\n{}", "Time entry started".green(), started_entry);

        Ok(())
    }
}
