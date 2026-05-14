use crate::api::client::ApiClient;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct RenameProjectCommand;

impl RenameProjectCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        old_name: String,
        new_name: String,
    ) -> ResultWithDefaultError<()> {
        let entities = api_client.get_entities().await?;

        let project = entities
            .projects
            .values()
            .find(|p| p.name == old_name)
            .cloned();

        match project {
            None => println!(
                "{}",
                format!("No project found with name '{old_name}'").yellow()
            ),
            Some(project) => {
                match api_client
                    .rename_project(project.workspace_id, project.id, new_name)
                    .await
                {
                    Err(error) => println!("{}\n{}", "Couldn't rename project".red(), error),
                    Ok(project) => {
                        println!("{}\n{}", "Project renamed successfully".green(), project)
                    }
                }
            }
        }

        Ok(())
    }
}
