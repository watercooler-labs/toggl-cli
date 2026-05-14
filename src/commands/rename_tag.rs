use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct RenameTagCommand;

impl RenameTagCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        old_name: String,
        new_name: String,
    ) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        let tags = api_client.get_tags(workspace_id).await?;

        let tag = tags.into_iter().find(|t| t.name == old_name);

        match tag {
            None => println!(
                "{}",
                format!("No tag found with name '{old_name}'").yellow()
            ),
            Some(tag) => match api_client.rename_tag(workspace_id, tag.id, new_name).await {
                Err(error) => println!("{}\n{}", "Couldn't rename tag".red(), error),
                Ok(tag) => println!("{}\n{}", "Tag renamed successfully".green(), tag),
            },
        }

        Ok(())
    }
}
