use crate::credentials;
use crate::error;
use crate::models;
use async_trait::async_trait;
use error::ApiError;
#[cfg(test)]
use mockall::automock;
use models::{ResultWithDefaultError, TimeEntry, User};
use reqwest::{header, Client, RequestBuilder, Response};
use serde::{de, Serialize};

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ApiClient {
    async fn get_user(&self) -> ResultWithDefaultError<User>;
    async fn get_running_time_entry(&self) -> ResultWithDefaultError<Option<TimeEntry>>;
    async fn get_time_entries(&self) -> ResultWithDefaultError<Vec<TimeEntry>>;
    async fn create_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<TimeEntry>;
    async fn update_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<TimeEntry>;
}

pub struct V9ApiClient {
    http_client: Client,
    base_url: String,
}

#[async_trait]
impl ApiClient for V9ApiClient {
    async fn get_user(&self) -> ResultWithDefaultError<User> {
        let url = format!("{}/me", self.base_url);
        return self.get::<User>(url).await;
    }

    async fn get_running_time_entry(&self) -> ResultWithDefaultError<Option<TimeEntry>> {
        let url = format!("{}/me/time_entries/current", self.base_url);
        return self.get::<Option<TimeEntry>>(url).await;
    }

    async fn get_time_entries(&self) -> ResultWithDefaultError<Vec<TimeEntry>> {
        let url = format!("{}/me/time_entries", self.base_url);
        return self.get::<Vec<TimeEntry>>(url).await;
    }

    async fn create_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<TimeEntry> {
        let url = format!("{}/time_entries", self.base_url);
        return self.post::<TimeEntry, TimeEntry>(url, &time_entry).await;
    }

    async fn update_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<TimeEntry> {
        let url = format!("{}/time_entries/{}", self.base_url, time_entry.id);
        return self.put::<TimeEntry, TimeEntry>(url, &time_entry).await;
    }
}

impl V9ApiClient {
    pub fn from_credentials(
        credentials: credentials::Credentials,
    ) -> ResultWithDefaultError<V9ApiClient> {
        let auth_string = credentials.api_token + ":api_token";
        let header_content = "Basic ".to_string() + base64::encode(auth_string).as_str();
        let mut headers = header::HeaderMap::new();
        let auth_header = header::HeaderValue::from_str(header_content.as_str())?;
        headers.insert(header::AUTHORIZATION, auth_header);

        let http_client = Client::builder().default_headers(headers).build()?;
        let api_client = Self {
            http_client,
            base_url: "https://track.toggl.com/api/v9".to_string(),
        };
        Ok(api_client)
    }

    async fn get<T: de::DeserializeOwned>(&self, url: String) -> ResultWithDefaultError<T> {
        let response = V9ApiClient::safe_send(self.http_client.get(url)).await?;
        let deserialized_json = V9ApiClient::safe_deserialize::<T>(response).await?;
        Ok(deserialized_json)
    }

    async fn put<T: de::DeserializeOwned, Body: Serialize>(
        &self,
        url: String,
        body: &Body,
    ) -> ResultWithDefaultError<T> {
        let response = V9ApiClient::safe_send(self.http_client.put(url).json(body)).await?;
        let deserialized_json = V9ApiClient::safe_deserialize::<T>(response).await?;
        Ok(deserialized_json)
    }

    async fn post<T: de::DeserializeOwned, Body: Serialize>(
        &self,
        url: String,
        body: &Body,
    ) -> ResultWithDefaultError<T> {
        let response = V9ApiClient::safe_send(self.http_client.post(url).json(body)).await?;
        let deserialized_json = V9ApiClient::safe_deserialize::<T>(response).await?;
        Ok(deserialized_json)
    }

    async fn safe_send(request: RequestBuilder) -> ResultWithDefaultError<Response> {
        match request.send().await {
            Err(_) => Err(Box::new(ApiError::Network)),
            Ok(response) => Ok(response),
        }
    }

    async fn safe_deserialize<T: de::DeserializeOwned>(
        response: Response,
    ) -> ResultWithDefaultError<T> {
        match response.json::<T>().await {
            Err(_) => Err(Box::new(ApiError::Deserialization)),
            Ok(response) => Ok(response),
        }
    }
}
