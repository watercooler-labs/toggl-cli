use crate::credentials;
use crate::models;
use chrono::Utc;
use models::{ResultWithDefaultError, TimeEntry, User};
use reqwest::{header, Client};
use serde::{de, Serialize};

pub struct ApiClient {
    http_client: Client,
    base_url: String,
}

impl ApiClient {
    pub fn from_credentials(
        credentials: credentials::Credentials,
    ) -> ResultWithDefaultError<ApiClient> {
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
        return Ok(api_client);
    }

    pub async fn get_user(&self) -> ResultWithDefaultError<User> {
        let url = format!("{}/me", self.base_url);
        return self.get::<User>(url).await;
    }

    pub async fn get_running_time_entry(&self) -> ResultWithDefaultError<Option<TimeEntry>> {
        let url = format!("{}/me/time_entries/current", self.base_url);
        return self.get::<Option<TimeEntry>>(url).await;
    }

    pub async fn get_time_entries(&self) -> ResultWithDefaultError<Vec<TimeEntry>> {
        let url = format!("{}/me/time_entries", self.base_url);
        return self.get::<Vec<TimeEntry>>(url).await;
    }

    pub async fn stop_time_entry(
        &self,
        time_entry: TimeEntry,
    ) -> ResultWithDefaultError<TimeEntry> {
        let mut stopped_time_entry = time_entry.clone();
        let stop_time = Utc::now();
        let duration = stop_time - stopped_time_entry.start;
        stopped_time_entry.stop = Some(stop_time);
        stopped_time_entry.duration = duration.num_seconds();

        let url = format!("{}/time_entries/{}", self.base_url, time_entry.id);
        return self
            .put::<TimeEntry, TimeEntry>(url, &stopped_time_entry)
            .await;
    }

    async fn get<T: de::DeserializeOwned>(&self, url: String) -> ResultWithDefaultError<T> {
        let result = self.http_client.get(url).send().await?;
        let deserialized_json = result.json::<T>().await?;
        return Ok(deserialized_json);
    }

    async fn put<T: de::DeserializeOwned, Body: Serialize>(
        &self,
        url: String,
        body: &Body,
    ) -> ResultWithDefaultError<T> {
        let result = self.http_client.put(url).json(body).send().await?;
        let deserialized_json = result.json::<T>().await?;
        return Ok(deserialized_json);
    }
}
