use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct DeleteTagCommand;

impl DeleteTagCommand {
    pub async fn execute(api_client: impl ApiClient, name: String) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        let tags = api_client.get_tags(workspace_id).await?;

        let tag = tags.into_iter().find(|t| t.name == name);

        match tag {
            None => println!("{}", format!("No tag found with name '{name}'").yellow()),
            Some(tag) => match api_client.delete_tag(workspace_id, tag.id).await {
                Err(error) => println!("{}\n{}", "Couldn't delete tag".red(), error),
                Ok(()) => println!("{}", "Tag deleted successfully".green()),
            },
        }

        Ok(())
    }
}
