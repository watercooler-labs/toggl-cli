use crate::credentials;
use crate::models;
use models::TimeEntry;
use models::User;
use reqwest::Client;
use reqwest::header;
use serde::de;

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
            base_url: "https://track.toggl.com/api/v9/".to_string()
        });
    }
    
    pub async fn get_user(&self) -> Result<User, Box<dyn std::error::Error>> {
        let url = self.base_url.to_owned() + "me";
        let result = self.get::<User>(url).await?;
        return Ok(result);
    }
    
    pub async fn get_time_entries(&self) -> Result<Vec<TimeEntry>, Box<dyn std::error::Error>> {
        let url = self.base_url.to_owned() + "me/time_entries";
        let result = self.get::<Vec<TimeEntry>>(url).await?;
        return Ok(result);
    }

    async fn get<T: de::DeserializeOwned>(&self, url: String) -> Result<T, Box<dyn std::error::Error>> {
        let result = self.client.get(url).send().await?;
        let deserialized_json = result.json::<T>().await?;
        return Ok(deserialized_json);
    }
}