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
                    .map_err(|e| {
                        println!("{}", e.to_string().red());
                        e
                    })
                    .unwrap();

                api_client.update_time_entry(updated_time_entry).await?;
            }
        }
        Ok(())
    }
}
