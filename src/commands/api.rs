use crate::api;
use crate::models::ResultWithDefaultError;
use api::client::ApiClient;
use serde_json::{Map, Value};

pub struct ApiCommand;

impl ApiCommand {
    pub async fn execute(
        api_client: impl ApiClient,
        path: &str,
        method: &str,
        raw_fields: &[String],
        typed_fields: &[String],
        input: Option<String>,
    ) -> ResultWithDefaultError<()> {
        let method = method.to_uppercase();
        let body = Self::build_body(raw_fields, typed_fields, input, &method).await?;
        let json = api_client.send_raw(path, &method, body).await?;
        if let Some(json) = json {
            println!(
                "{}",
                serde_json::to_string_pretty(&json).expect("failed to serialize response")
            );
        }
        Ok(())
    }

    /// Builds the JSON request body from `--raw-field`/`--field` entries or
    /// `--input`. Returns `None` when no body was requested.
    async fn build_body(
        raw_fields: &[String],
        typed_fields: &[String],
        input: Option<String>,
        method: &str,
    ) -> ResultWithDefaultError<Option<Value>> {
        if input.is_some() && (!raw_fields.is_empty() || !typed_fields.is_empty()) {
            return Err(Box::new(std::io::Error::other(
                "cannot combine --input with --raw-field or --field",
            )));
        }
        if method == "GET"
            && (!raw_fields.is_empty() || !typed_fields.is_empty() || input.is_some())
        {
            return Err(Box::new(std::io::Error::other(
                "GET requests do not take a body; pass -X to use a different method",
            )));
        }
        if let Some(input) = input {
            let input = Self::read_source(&input).await?;
            return Ok(Some(Self::validate_input(input)?));
        }
        if raw_fields.is_empty() && typed_fields.is_empty() {
            return Ok(None);
        }
        let mut map = Map::new();
        for field in raw_fields {
            let (key, value) = Self::parse_field("--raw-field", field)?;
            map.insert(key, Value::String(value));
        }
        for field in typed_fields {
            let (key, value) = Self::parse_field("--field", field)?;
            map.insert(key, Self::typed_value(&value).await?);
        }
        Ok(Some(Value::Object(map)))
    }

    fn parse_field(flag: &str, field: &str) -> ResultWithDefaultError<(String, String)> {
        let (key, value) =
            field
                .split_once('=')
                .ok_or_else(|| -> Box<dyn std::error::Error + Send> {
                    Box::new(std::io::Error::other(format!(
                        "invalid {flag} \"{field}\": expected key=value"
                    )))
                })?;
        if key.is_empty() {
            return Err(Box::new(std::io::Error::other(format!(
                "invalid {flag} \"{field}\": empty key"
            ))));
        }
        Ok((key.to_string(), value.to_string()))
    }

    /// Resolves a typed field value: `@<path>` and `@-` read the value from
    /// a file or stdin; otherwise the value is parsed inline as a boolean,
    /// null, integer, float, or string.
    async fn typed_value(value: &str) -> ResultWithDefaultError<Value> {
        match value.strip_prefix('@') {
            Some(source) => {
                let raw = Self::read_source(source).await?;
                Ok(Self::parse_typed(&raw))
            }
            None => Ok(Self::parse_typed(value)),
        }
    }

    /// Parses a typed field value: booleans, null, integers and floats are
    /// represented as such; anything else stays a string.
    fn parse_typed(value: &str) -> Value {
        match value {
            "true" => return Value::Bool(true),
            "false" => return Value::Bool(false),
            "null" => return Value::Null,
            _ => {}
        }
        if let Ok(integer) = value.parse::<i64>() {
            return Value::Number(integer.into());
        }
        if let Ok(float) = value.parse::<f64>() {
            if let Some(number) = serde_json::Number::from_f64(float) {
                return Value::Number(number);
            }
        }
        Value::String(value.to_string())
    }

    /// Reads a body or field value from a file path or `-` for stdin.
    /// Blocking reads run on the blocking thread pool so they never park an
    /// async worker.
    async fn read_source(source: &str) -> ResultWithDefaultError<String> {
        let raw = if source == "-" {
            tokio::task::spawn_blocking(|| -> std::io::Result<String> {
                use std::io::Read;
                let mut buffer = String::new();
                std::io::stdin().read_to_string(&mut buffer)?;
                Ok(buffer)
            })
            .await
        } else {
            let path = source.to_string();
            tokio::task::spawn_blocking(move || std::fs::read_to_string(&path)).await
        }
        .map_err(|error| -> Box<dyn std::error::Error + Send> { Box::new(error) })?
        .map_err(|error| -> Box<dyn std::error::Error + Send> {
            if source == "-" {
                Box::new(error)
            } else {
                Box::new(std::io::Error::other(format!(
                    "cannot read {source}: {error}"
                )))
            }
        })?;
        Ok(raw)
    }

    fn validate_input(raw: String) -> ResultWithDefaultError<Value> {
        serde_json::from_str::<Value>(&raw).map_err(|error| -> Box<dyn std::error::Error + Send> {
            Box::new(std::io::Error::other(format!(
                "invalid JSON in --input: {error}"
            )))
        })
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
        api_client.expect_send_raw().returning(|_, _, _| {
            Ok(Some(serde_json::json!({
                "id": 1,
                "description": "raw entry",
                "duration": -1795107819
            })))
        });
        api_client
    }

    fn create_failing_api_client() -> MockApiClient {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_send_raw()
            .returning(|_, _, _| Err(Box::new(ApiError::Network)));
        api_client
    }

    #[tokio::test]
    async fn test_accepts_valid_path() {
        let result = ApiCommand::execute(
            create_working_api_client(),
            "/me/time_entries/current",
            "GET",
            &[],
            &[],
            None,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn test_propagates_api_errors() {
        let result =
            ApiCommand::execute(create_failing_api_client(), "/me", "GET", &[], &[], None).await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn test_rejects_field_on_get() {
        let result = ApiCommand::execute(
            create_working_api_client(),
            "/me",
            "GET",
            &["name=test".to_string()],
            &[],
            None,
        )
        .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn test_rejects_typed_field_on_get() {
        let result = ApiCommand::execute(
            create_working_api_client(),
            "/me",
            "GET",
            &[],
            &["billable=true".to_string()],
            None,
        )
        .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn test_rejects_field_and_input_together() {
        let result = ApiCommand::execute(
            create_working_api_client(),
            "/me",
            "POST",
            &["name=test".to_string()],
            &[],
            Some("body.json".to_string()),
        )
        .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn test_rejects_typed_field_and_input_together() {
        let result = ApiCommand::execute(
            create_working_api_client(),
            "/me",
            "POST",
            &[],
            &["wid=1".to_string()],
            Some("body.json".to_string()),
        )
        .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn test_rejects_field_without_equals() {
        let result = ApiCommand::execute(
            create_working_api_client(),
            "/me",
            "POST",
            &["name".to_string()],
            &[],
            None,
        )
        .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn test_rejects_field_with_empty_key() {
        let result = ApiCommand::execute(
            create_working_api_client(),
            "/me",
            "POST",
            &["=value".to_string()],
            &[],
            None,
        )
        .await;
        assert_err!(result);
    }

    #[tokio::test]
    async fn test_sends_raw_field_values_as_strings() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_send_raw()
            .withf(|path, method, body| {
                path == "/workspaces/1/projects"
                    && method == "POST"
                    && *body
                        == Some(serde_json::json!({
                            "name": "test",
                            "color": "#06aaf5",
                            "billable": "true"
                        }))
            })
            .returning(|_, _, _| Ok(Some(serde_json::json!({"id": 1}))));

        let result = ApiCommand::execute(
            api_client,
            "/workspaces/1/projects",
            "POST",
            &[
                "name=test".to_string(),
                "color=#06aaf5".to_string(),
                "billable=true".to_string(),
            ],
            &[],
            None,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn test_sends_typed_field_values() {
        let mut api_client = MockApiClient::new();
        api_client
            .expect_send_raw()
            .withf(|path, method, body| {
                path == "/workspaces/1/projects"
                    && method == "POST"
                    && *body
                        == Some(serde_json::json!({
                            "wid": 123,
                            "billable": true,
                            "private": false,
                            "premium": null,
                            "rate": 55.5,
                            "description": "deep work"
                        }))
            })
            .returning(|_, _, _| Ok(Some(serde_json::json!({"id": 1}))));

        let result = ApiCommand::execute(
            api_client,
            "/workspaces/1/projects",
            "POST",
            &[],
            &[
                "wid=123".to_string(),
                "billable=true".to_string(),
                "private=false".to_string(),
                "premium=null".to_string(),
                "rate=55.5".to_string(),
                "description=deep work".to_string(),
            ],
            None,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn test_sends_typed_field_value_from_file() {
        use std::io::Write;

        let mut file = tempfile::NamedTempFile::new().expect("failed to create temp file");
        write!(file, "true").expect("failed to write temp file");
        let path = file.path().to_string_lossy().to_string();

        let mut api_client = MockApiClient::new();
        api_client
            .expect_send_raw()
            .withf(move |_, _, body| *body == Some(serde_json::json!({"billable": true})))
            .returning(|_, _, _| Ok(Some(serde_json::json!({"id": 1}))));

        let result = ApiCommand::execute(
            api_client,
            "/me",
            "POST",
            &[],
            &[format!("billable=@{path}")],
            None,
        )
        .await;
        assert_ok!(result);
    }

    #[tokio::test]
    async fn test_rejects_typed_field_from_missing_file() {
        let result = ApiCommand::execute(
            create_working_api_client(),
            "/me",
            "POST",
            &[],
            &["billable=@/nonexistent/path".to_string()],
            None,
        )
        .await;
        assert_err!(result);
    }
}

#[cfg(test)]
mod typed_parsing_tests {
    use super::ApiCommand;
    use serde_json::json;

    #[test]
    fn parses_booleans() {
        assert_eq!(ApiCommand::parse_typed("true"), json!(true));
        assert_eq!(ApiCommand::parse_typed("false"), json!(false));
    }

    #[test]
    fn parses_null() {
        assert_eq!(ApiCommand::parse_typed("null"), json!(null));
    }

    #[test]
    fn parses_integers() {
        assert_eq!(ApiCommand::parse_typed("123"), json!(123));
        assert_eq!(ApiCommand::parse_typed("-7"), json!(-7));
    }

    #[test]
    fn parses_floats() {
        assert_eq!(ApiCommand::parse_typed("55.5"), json!(55.5));
    }

    #[test]
    fn keeps_unparseable_values_as_strings() {
        assert_eq!(ApiCommand::parse_typed("deep work"), json!("deep work"));
        assert_eq!(ApiCommand::parse_typed("#06aaf5"), json!("#06aaf5"));
        assert_eq!(ApiCommand::parse_typed("2026-11-19"), json!("2026-11-19"));
    }
}
