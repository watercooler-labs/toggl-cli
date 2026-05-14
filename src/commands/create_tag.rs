use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct CreateTagCommand;

impl CreateTagCommand {
    pub async fn execute(api_client: impl ApiClient, name: String) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        match api_client.create_tag(workspace_id, name).await {
            Err(error) => println!("{}\n{}", "Couldn't create tag".red(), error),
            Ok(tag) => println!("{}\n{}", "Tag created successfully".green(), tag),
        }
        Ok(())
    }
}
