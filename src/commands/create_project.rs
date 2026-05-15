use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct CreateProjectCommand;

impl CreateProjectCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        name: String,
        color: String,
    ) -> ResultWithDefaultError<()> {
        let workspace_id = api_client.get_user().await?.default_workspace_id;
        match api_client.create_project(workspace_id, name, color).await {
            Err(error) => println!("{}\n{}", "Couldn't create project".red(), error),
            Ok(project) => println!("{}\n{}", "Project created successfully".green(), project),
        }
        Ok(())
    }
}
