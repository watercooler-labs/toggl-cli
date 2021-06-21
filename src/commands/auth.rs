use crate::credentials;
use crate::models;
use crate::api;
use api::ApiClient;
use colored::Colorize;
use credentials::CredentialsStorage;
use models::ResultWithDefaultError;

pub struct AuthenticationCommand;

impl AuthenticationCommand {
    pub async fn execute(api_client: impl ApiClient, credentials_storage: impl CredentialsStorage) -> ResultWithDefaultError<()> {
        let user = api_client.get_user().await?;
        credentials_storage.persist(user.api_token)?;
        println!(
            "{} {}",
            "Successfully authenticated for user with email:".green(),
            user.email.green().bold(),
        );

        return Ok(());
    }
}