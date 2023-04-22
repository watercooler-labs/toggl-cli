use crate::api;
use crate::models;
use api::client::ApiClient;
use chrono::Utc;
use colored::Colorize;
use models::{ResultWithDefaultError, TimeEntry};

pub struct StopCommand;

pub enum StopCommandOrigin {
    CommandLine,
    StartCommand,
    ContinueCommand,
}

impl StopCommand {
    pub async fn execute(
        api_client: &impl ApiClient,
        origin: StopCommandOrigin,
    ) -> ResultWithDefaultError<Option<TimeEntry>> {
        match api_client.get_running_time_entry().await? {
            None => {
                match origin {
                    StopCommandOrigin::CommandLine => {
                        println!("{}", "No time entry is running at the moment".yellow())
                    }
                    StopCommandOrigin::StartCommand => (),
                    StopCommandOrigin::ContinueCommand => (),
                };

                Ok(None)
            }
            Some(running_time_entry) => {
                let stop_time = Utc::now();
                let stopped_time_entry = running_time_entry.as_stopped_time_entry(stop_time);
                let updated_time_entry = api_client.update_time_entry(stopped_time_entry).await?;

                let message = match origin {
                    StopCommandOrigin::CommandLine => "Time entry stopped successfully".green(),
                    StopCommandOrigin::StartCommand => "Running time entry stopped".yellow(),
                    StopCommandOrigin::ContinueCommand => "Running time entry stopped".yellow(),
                };

                println!("{}\n{}", message, updated_time_entry);

                Ok(Some(updated_time_entry))
            }
        }
    }
}
