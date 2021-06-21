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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error;
    use api::MockApiClient;
    use credentials::MockCredentialsStorage;
    use models::User;
    use error::{ApiError, StorageError};
    use tokio_test::assert_ok;
    use tokio_test::assert_err;

    const MOCK_API_TOKEN: &str = "SOME_API_TOKEN_VALUE";
    const MOCK_EMAIL: &str = "toggl@user.org";
    const MOCK_USER_NAME: &str = "Toggl User";

    fn create_working_api_client() -> MockApiClient {
        let mut api_client = MockApiClient::new();
        let user = User { 
            api_token: MOCK_API_TOKEN.to_string(),
            email: MOCK_EMAIL.to_string(),
            fullname: Some(MOCK_USER_NAME.to_string()),
            timezone: "UTC".to_string(),
            default_workspace_id: 1
        };

        api_client.expect_get_user().returning(move ||Ok(user.clone()));

        return api_client;
    }

    fn create_failing_api_client() -> MockApiClient {
        let mut api_client = MockApiClient::new();
        api_client.expect_get_user().returning(||Err(Box::new(ApiError::Network)));

        return api_client;
    }

    fn create_working_credentials_storage() -> MockCredentialsStorage {
        let mut credentials_storage = MockCredentialsStorage::new();
        credentials_storage.expect_persist().returning(|_|Ok(()));
        return credentials_storage;
    }

    fn create_failing_credentials_storage() -> MockCredentialsStorage {
        let mut credentials_storage = MockCredentialsStorage::new();
        credentials_storage.expect_persist().returning(|_|Err(Box::new(StorageError::Write)));
        return credentials_storage;
    }

    #[tokio::test]
    async fn a_valid_api_call_and_a_working_storage_returns_an_ok_result() {
        // Arrange
        let mut output = Vec::new();
        let api_client = create_working_api_client();
        let credentials_storage = create_working_credentials_storage();

        // Act
        let result = AuthenticationCommand::execute(&mut output, api_client, credentials_storage).await;

        // Assert
        assert_ok!(result);
    }


    #[tokio::test]
    async fn a_valid_api_call_and_a_working_storage_prints_a_success_message() {
        // Arrange
        let mut output = Vec::new();
        let api_client = create_working_api_client();
        let credentials_storage = create_working_credentials_storage();

        // Act
        let _ = AuthenticationCommand::execute(&mut output, api_client, credentials_storage).await;
    
        // Assert
        let expected_output = format!(
            "{} {}",
            AUTH_SUCCEEDED_MESSAGE.green(),
            MOCK_EMAIL.green().bold()
        );
        let actual_output = String::from_utf8(output).expect(format!("empty output when {} was expected", expected_output).as_str());

        assert_eq!(expected_output, actual_output);
    }

    #[tokio::test]
    async fn a_failing_api_call_returns_an_error() {
        // Arrange
        let mut output = Vec::new();
        let api_client = create_failing_api_client();
        let credentials_storage = create_working_credentials_storage();
        
        // Act
        let result = AuthenticationCommand::execute(&mut output, api_client, credentials_storage).await;
        
        // Assert
        assert_err!(result);
    }

    #[tokio::test]
    async fn a_failing_storage_persist_call_returns_an_error() {
        // Arrange
        let mut output = Vec::new();
        let api_client = create_working_api_client();
        let credentials_storage = create_failing_credentials_storage();
        
        // Act
        let result = AuthenticationCommand::execute(&mut output, api_client, credentials_storage).await;
        
        // Assert
        assert_err!(result);
    }
}