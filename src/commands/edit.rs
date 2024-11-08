use crate::api::client::ApiClient;
use crate::models;
use crate::parcel::Parcel;
use crate::picker;
use colored::Colorize;
use models::ResultWithDefaultError;
use picker::ItemPicker;

pub struct EditCommand;

impl EditCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        _picker: Option<Box<dyn ItemPicker>>,
        _interactive: bool,
    ) -> ResultWithDefaultError<()> {
        let entities = api_client.get_entities().await?;
        match entities.running_time_entry() {
            None => println!("{}", "No time entry is running at the moment".yellow()),
            Some(running_time_entry) => {
                let updated_time_entry = running_time_entry
                    .launch_in_editor()
                    .inspect_err(|e| {
                        println!("{}", e.to_string().red());
                    })
                    .unwrap();

                let updated_entry_id = api_client
                    .update_time_entry(updated_time_entry.clone())
                    .await;
                if updated_entry_id.is_err() {
                    println!("{}", "Failed to update time entry".red());
                    return Err(updated_entry_id.err().unwrap());
                }

                println!("{}\n{}", "Time entry updated".green(), updated_time_entry);
                return Ok(());
            }
        }
        Ok(())
    }
}
