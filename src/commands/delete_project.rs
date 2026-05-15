use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct DeleteProjectCommand;

impl DeleteProjectCommand {
    pub async fn execute(api_client: impl ApiClient, name: String) -> ResultWithDefaultError<()> {
        let entities = api_client.get_entities().await?;

        let project = entities.projects.values().find(|p| p.name == name).cloned();

        match project {
            None => println!(
                "{}",
                format!("No project found with name '{name}'").yellow()
            ),
            Some(project) => {
                match api_client
                    .delete_project(project.workspace_id, project.id)
                    .await
                {
                    Err(error) => println!("{}\n{}", "Couldn't delete project".red(), error),
                    Ok(()) => println!("{}\n{}", "Project deleted successfully".green(), project),
                }
            }
        }

        Ok(())
    }
}
