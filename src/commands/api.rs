use crate::api;
use crate::models::ResultWithDefaultError;
use api::client::ApiClient;

pub struct ApiCommand;

impl ApiCommand {
    pub async fn execute(api_client: impl ApiClient, path: &str) -> ResultWithDefaultError<()> {
        if !path.starts_with('/') || path.starts_with("//") {
            return Err(Box::new(std::io::Error::other(format!(
                "invalid API path \"{path}\": expected a path starting with a single '/', \
                 e.g. \"/me/time_entries/current\""
            ))));
        }
        if path.contains("://") {
            return Err(Box::new(std::io::Error::other(format!(
                "invalid API path \"{path}\": pass a relative path, not a full URL"
            ))));
        }
        let json = api_client.get_raw(path).await?;
        println!(
            "{}",
            serde_json::to_string_pretty(&json).expect("failed to serialize response")
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::api::client::MockApiClient;
    use crate::error;
    use error::ApiError;
    use tokio_test::{assert_err, assert_ok};

    fn create_working_api_client() -> MockApiClient {
        let mut api_client = MockApiClient::new();
        api_client.expect_get_raw().returning(|_| {
            Ok(serde_json::json!({
                "id": 1,
                "description": "raw entry",
                "duration": -1795107819
            }))
        });
        api_client
    }

    fn create_failing_api_client() -> MockApiClient {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_get_raw()
            .returning(|_| Err(Box::new(ApiError::Network)));
        api_client
    }

    #[tokio::test]
    async fn test_accepts_valid_path() {
        let result = ApiCommand::execute(create_working_api_client(), "/me/time_entries/current")
            .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn test_rejects_path_without_leading_slash() {
        let result = ApiCommand::execute(create_working_api_client(), "me/time_entries").await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn test_rejects_double_slash_path() {
        let result = ApiCommand::execute(create_working_api_client(), "//evil.com/me").await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn test_rejects_full_url() {
        let result =
            ApiCommand::execute(create_working_api_client(), "https://example.com/me").await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn test_propagates_api_errors() {
        let result = ApiCommand::execute(create_failing_api_client(), "/me").await;
        assert_err!(result);
    }
}
