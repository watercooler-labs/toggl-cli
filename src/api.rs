use crate::credentials;
use crate::models;
use chrono::Utc;
use models::{TimeEntry, User};
use reqwest::{Client, header};
use serde::{de, Serialize};

pub struct ApiClient {
    client: Client,
    base_url: String
}

impl ApiClient {
    pub fn from_credentials(credentials: credentials::Credentials) -> Result<ApiClient, Box<dyn std::error::Error>> {
        
        let auth_string = credentials.api_token + ":api_token";
        let header_content = "Basic ".to_string() + base64::encode(auth_string).as_str();
        let mut headers = header::HeaderMap::new();
        let auth_header = header::HeaderValue::from_str(header_content.as_str())?;
        headers.insert(header::AUTHORIZATION, auth_header);

        let client = Client::builder().default_headers(headers).build()?;
        return Ok(Self {
            client: client,
            base_url: "https://track.toggl.com/api/v9".to_string()
        });
    }
    
    pub async fn get_user(&self) -> Result<User, Box<dyn std::error::Error>> {
        let url = format!("{}/me", self.base_url);
        let result = self.get::<User>(url).await?;
        return Ok(result);
    }
    
    pub async fn get_time_entries(&self) -> Result<Vec<TimeEntry>, Box<dyn std::error::Error>> {
        let url = format!("{}/me/time_entries", self.base_url);
        let result = self.get::<Vec<TimeEntry>>(url).await?;
        return Ok(result);
    }

    pub async fn stop_time_entry(&self, time_entry: TimeEntry) -> Result<TimeEntry, Box<dyn std::error::Error>> {
        let mut stopped_time_entry = time_entry.clone();
        let stop_time = Utc::now();
        let duration = stop_time - stopped_time_entry.start;
        stopped_time_entry.stop = Some(stop_time);
        stopped_time_entry.duration = duration.num_seconds();

        let url = format!("{}/time_entries/{}", self.base_url, time_entry.id);
        let result = self.put::<TimeEntry, TimeEntry>(url, &stopped_time_entry).await?;

        return Ok(result);
    }

    async fn get<T: de::DeserializeOwned>(&self, url: String) -> Result<T, Box<dyn std::error::Error>> {
        let result = self.client.get(url).send().await?;
        let deserialized_json = result.json::<T>().await?;
        return Ok(deserialized_json);
    }
    async fn put<T: de::DeserializeOwned, Body: Serialize>(&self, url: String, body: &Body) -> Result<T, Box<dyn std::error::Error>> {
        let result = self.client.put(url).json(body).send().await?;
        let deserialized_json = result.json::<T>().await?;
        return Ok(deserialized_json);
    }

}