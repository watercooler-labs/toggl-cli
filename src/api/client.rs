use std::collections::HashMap;

use crate::credentials;
use crate::error;
use crate::models;
use crate::models::Entities;
use crate::models::Project;
use crate::models::Task;
use crate::models::TimeEntry;
use crate::models::Workspace;
use async_trait::async_trait;
use base64::{engine::general_purpose, Engine as _};
use error::ApiError;
#[cfg(test)]
use mockall::automock;
use models::{ResultWithDefaultError, User};
use reqwest::Client;
use reqwest::{header, RequestBuilder};
use serde::{de, Serialize};

use super::models::NetworkClient;
use super::models::NetworkProject;
use super::models::NetworkTask;
use super::models::NetworkTimeEntry;
use super::models::NetworkWorkspace;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ApiClient {
    async fn get_user(&self) -> ResultWithDefaultError<User>;
    async fn get_entities(&self) -> ResultWithDefaultError<Entities>;

    async fn create_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<i64>;
    async fn update_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<i64>;
}

pub struct V9ApiClient {
    http_client: Client,
    base_url: String,
}

impl V9ApiClient {
    async fn get_time_entries(&self) -> ResultWithDefaultError<Vec<NetworkTimeEntry>> {
        let url = format!("{}/me/time_entries", self.base_url);
        self.get::<Vec<NetworkTimeEntry>>(url).await
    }

    async fn get_projects(&self) -> ResultWithDefaultError<Vec<NetworkProject>> {
        let url = format!("{}/me/projects", self.base_url);
        self.get::<Vec<NetworkProject>>(url).await
    }

    async fn get_clients(&self) -> ResultWithDefaultError<Vec<NetworkClient>> {
        let url = format!("{}/me/clients", self.base_url);
        self.get::<Vec<NetworkClient>>(url).await
    }

    async fn get_tasks(&self) -> ResultWithDefaultError<Vec<NetworkTask>> {
        let url = format!("{}/me/tasks", self.base_url);
        self.get::<Vec<NetworkTask>>(url).await
    }

    async fn get_workspaces(&self) -> ResultWithDefaultError<Vec<NetworkWorkspace>> {
        let url = format!("{}/me/workspaces", self.base_url);
        self.get::<Vec<NetworkWorkspace>>(url).await
    }

    pub fn from_credentials(
        credentials: credentials::Credentials,
        proxy: Option<String>,
    ) -> ResultWithDefaultError<V9ApiClient> {
        let auth_string = credentials.api_token + ":api_token";
        let header_content =
            "Basic ".to_string() + general_purpose::STANDARD.encode(auth_string).as_str();
        let mut headers = header::HeaderMap::new();
        let auth_header =
            header::HeaderValue::from_str(header_content.as_str()).expect("Invalid header value");
        headers.insert(header::AUTHORIZATION, auth_header);

        let base_client = Client::builder().default_headers(headers);
        let http_client = {
            if let Some(proxy) = proxy {
                base_client.proxy(reqwest::Proxy::all(proxy).expect("Invalid proxy"))
            } else {
                base_client
            }
        }
        .build()
        .expect("Couldn't build a http client");
        let api_client = Self {
            http_client,
            base_url: "https://track.toggl.com/api/v9".to_string(),
        };
        Ok(api_client)
    }

    async fn get<T: de::DeserializeOwned>(&self, url: String) -> ResultWithDefaultError<T> {
        V9ApiClient::send::<T>(self.http_client.get(url)).await
    }

    async fn put<T: de::DeserializeOwned, Body: Serialize>(
        &self,
        url: String,
        body: &Body,
    ) -> ResultWithDefaultError<T> {
        V9ApiClient::send::<T>(self.http_client.put(url).json(body)).await
    }

    async fn post<T: de::DeserializeOwned, Body: Serialize>(
        &self,
        url: String,
        body: &Body,
    ) -> ResultWithDefaultError<T> {
        V9ApiClient::send::<T>(self.http_client.post(url).json(body)).await
    }

    async fn send<T: de::DeserializeOwned>(request: RequestBuilder) -> ResultWithDefaultError<T> {
        match request.send().await {
            Err(_) => Err(Box::new(ApiError::Network)),
            Ok(response) => match response.json::<T>().await {
                Err(_) => Err(Box::new(ApiError::Deserialization)),
                Ok(parsed_response) => Ok(parsed_response),
            },
        }
    }
}

#[async_trait]
impl ApiClient for V9ApiClient {
    async fn get_user(&self) -> ResultWithDefaultError<User> {
        let url = format!("{}/me", self.base_url);
        return self.get::<User>(url).await;
    }

    async fn create_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<i64> {
        let url = format!("{}/time_entries", self.base_url);
        let network_time_entry = self
            .post::<NetworkTimeEntry, NetworkTimeEntry>(url, &time_entry.into())
            .await?;
        return Ok(network_time_entry.id);
    }

    async fn update_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<i64> {
        let url = format!("{}/time_entries/{}", self.base_url, time_entry.id);
        let network_time_entry = self
            .put::<NetworkTimeEntry, NetworkTimeEntry>(url, &time_entry.into())
            .await?;
        return Ok(network_time_entry.id);
    }

    async fn get_entities(&self) -> ResultWithDefaultError<Entities> {
        let (
            network_time_entries,
            network_projects,
            network_tasks,
            network_clients,
            network_workspaces,
        ) = tokio::join!(
            self.get_time_entries(),
            self.get_projects(),
            self.get_tasks(),
            self.get_clients(),
            self.get_workspaces(),
        );

        let clients: HashMap<i64, crate::models::Client> = network_clients
            .unwrap_or_default()
            .iter()
            .map(|c| {
                (
                    c.id,
                    crate::models::Client {
                        id: c.id,
                        name: c.name.clone(),
                        workspace_id: c.wid,
                    },
                )
            })
            .collect();

        let projects: HashMap<i64, Project> = network_projects
            .unwrap_or_default()
            .iter()
            .map(|p| {
                (
                    p.id,
                    Project {
                        id: p.id,
                        name: p.name.clone(),
                        workspace_id: p.workspace_id,
                        client: clients.get(&p.client_id.unwrap_or(-1)).cloned(),
                        is_private: p.is_private,
                        active: p.active,
                        at: p.at,
                        created_at: p.created_at,
                        color: p.color.clone(),
                        billable: p.billable,
                    },
                )
            })
            .collect();

        let tasks: HashMap<i64, Task> = network_tasks
            .unwrap_or_default()
            .iter()
            .map(|t| {
                (
                    t.id,
                    Task {
                        id: t.id,
                        name: t.name.clone(),
                        project: projects.get(&t.project_id).unwrap().clone(),
                        workspace_id: t.workspace_id,
                    },
                )
            })
            .collect();

        let time_entries = network_time_entries
            .unwrap_or_default()
            .iter()
            .map(|te| TimeEntry {
                id: te.id,
                description: te.description.clone(),
                start: te.start,
                stop: te.stop,
                duration: te.duration,
                billable: te.billable,
                workspace_id: te.workspace_id,
                tags: te.tags.clone(),
                project: projects.get(&te.project_id.unwrap_or(-1)).cloned(),
                task: tasks.get(&te.task_id.unwrap_or(-1)).cloned(),
                ..Default::default()
            })
            .collect();

        let workspaces = network_workspaces
            .unwrap_or_default()
            .iter()
            .map(|w| Workspace {
                id: w.id,
                name: w.name.clone(),
                admin: w.admin,
            })
            .collect();

        Ok(Entities {
            time_entries,
            projects,
            tasks,
            clients,
            workspaces,
        })
    }
}
