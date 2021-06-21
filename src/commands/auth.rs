use crate::api;
use crate::credentials;
use crate::models;
use api::ApiClient;
use colored::Colorize;
use credentials::CredentialsStorage;
use models::ResultWithDefaultError;
use std::io::Write;

pub struct AuthenticationCommand;

const AUTH_SUCCEEDED_MESSAGE: &str = "Successfully authenticated for user with email:";

impl AuthenticationCommand {
    pub async fn execute<W: Write>(
        mut writer: W,
        api_client: impl ApiClient,
        credentials_storage: impl CredentialsStorage,
    ) -> ResultWithDefaultError<()> {
        let user = api_client.get_user().await?;
        credentials_storage.persist(user.api_token)?;
        write!(
            writer,
            "{} {}",
            AUTH_SUCCEEDED_MESSAGE.green(),
            user.email.green().bold(),
        )?;

        Ok(())
    }
}
