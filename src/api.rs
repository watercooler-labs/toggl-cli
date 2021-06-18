use crate::credentials;
use crate::models;
use reqwest::Client;
use reqwest::header;

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
    
