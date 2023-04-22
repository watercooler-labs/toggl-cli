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
        let entities = api_client.get_entities().await?;
        match entities.running_time_entry() {
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
                api_client
                    .update_time_entry(stopped_time_entry.clone())
                    .await?;

                let message = match origin {
                    StopCommandOrigin::CommandLine => "Time entry stopped successfully".green(),
                    StopCommandOrigin::StartCommand => "Running time entry stopped".yellow(),
                    StopCommandOrigin::ContinueCommand => "Running time entry stopped".yellow(),
                };

                println!("{}\n{}", message, stopped_time_entry.clone());

                Ok(Some(stopped_time_entry))
            }
        }
    }
}
