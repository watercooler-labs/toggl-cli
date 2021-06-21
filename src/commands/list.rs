use crate::models;
use crate::api;
use api::ApiClient;
use colored::Colorize;
use models::ResultWithDefaultError;

pub struct ListCommand;

impl ListCommand {
    pub async fn execute(api_client: impl ApiClient, count: Option<usize>) -> ResultWithDefaultError<()> {
        match api_client.get_time_entries().await {
            Err(error) => println!(
                "{}\n{}",
                "Couldn't fetch time entries the from API".red(),
                error
            ),
            Ok(time_entries) => time_entries
                .iter()
                .take(count.unwrap_or(usize::max_value()))
                .for_each(|time_entry| println!("{}", time_entry)),
        }

        Ok(())
    }
}