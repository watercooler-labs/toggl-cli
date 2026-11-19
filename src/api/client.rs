use std::collections::HashMap;

use crate::credentials;
use crate::error;
use crate::models;
use crate::models::Entities;
use crate::models::Project;
use crate::models::Tag;
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
use serde_json::Value;

use super::models::NetworkClient;
use super::models::NetworkCreateProject;
use super::models::NetworkCreateTag;
use super::models::NetworkProject;
use super::models::NetworkRenameProject;
use super::models::NetworkRenameTag;
use super::models::NetworkTag;
use super::models::NetworkTask;
use super::models::NetworkTimeEntry;
use super::models::NetworkWorkspace;

#[cfg_attr(test, automock)]
#[async_trait]
pub trait ApiClient {
    async fn get_user(&self) -> ResultWithDefaultError<User>;

    /// Issues a raw request against the API and returns the JSON response,
    /// without mapping it into domain models. Used for debugging.
    /// Returns `None` when the response has an empty body (e.g. HTTP 204).
    async fn send_raw(
        &self,
        path: &str,
        method: &str,
        body: Option<Value>,
    ) -> ResultWithDefaultError<Option<Value>>;
    async fn get_entities(&self) -> ResultWithDefaultError<Entities>;

    async fn create_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<i64>;
    async fn update_time_entry(&self, time_entry: TimeEntry) -> ResultWithDefaultError<i64>;
    async fn get_time_entries_filtered(
        &self,
        since: Option<String>,
        until: Option<String>,
    ) -> ResultWithDefaultError<Vec<TimeEntry>>;
    async fn delete_time_entry(
        &self,
        workspace_id: i64,
        time_entry_id: i64,
    ) -> ResultWithDefaultError<()>;

    async fn create_project(
        &self,
        workspace_id: i64,
        name: String,
        color: String,
    ) -> ResultWithDefaultError<Project>;

    async fn delete_project(
        &self,
        workspace_id: i64,
        project_id: i64,
    ) -> ResultWithDefaultError<()>;

    async fn rename_project(
        &self,
        workspace_id: i64,
        project_id: i64,
        new_name: String,
    ) -> ResultWithDefaultError<Project>;

    async fn get_tags(&self, workspace_id: i64) -> ResultWithDefaultError<Vec<Tag>>;

    async fn create_tag(&self, workspace_id: i64, name: String) -> ResultWithDefaultError<Tag>;

    async fn delete_tag(&self, workspace_id: i64, tag_id: i64) -> ResultWithDefaultError<()>;

    async fn rename_tag(
        &self,
        workspace_id: i64,
        tag_id: i64,
        new_name: String,
    ) -> ResultWithDefaultError<Tag>;
}

pub struct V9ApiClient {
    http_client: Client,
    base_url: String,
}

impl V9ApiClient {
    async fn get_time_entries(
        &self,
        since: Option<&str>,
        until: Option<&str>,
    ) -> ResultWithDefaultError<Vec<NetworkTimeEntry>> {
        let mut url = format!("{}/me/time_entries", self.base_url);
        let mut params: Vec<String> = Vec::new();
        if let Some(since) = since {
            params.push(format!("start_date={since}"));
        }
        if let Some(until) = until {
            params.push(format!("end_date={until}"));
        }
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }
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

    async fn get_workspace_tags(
        &self,
        workspace_id: i64,
    ) -> ResultWithDefaultError<Vec<NetworkTag>> {
        let url = format!("{}/workspaces/{}/tags", self.base_url, workspace_id);
        self.get::<Vec<NetworkTag>>(url).await
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

    async fn send_raw_request(
        &self,
        path: &str,
        method: &str,
        body: Option<Value>,
    ) -> ResultWithDefaultError<Option<Value>> {
        let url = resolve_url(&self.base_url, path)?;
        let method = method.to_uppercase().parse::<reqwest::Method>().map_err(
            |_| -> Box<dyn std::error::Error + Send> {
                Box::new(std::io::Error::other(format!(
                    "invalid HTTP method: {method}"
                )))
            },
        )?;
        let request = self.http_client.request(method, url);
        let request = match body {
            Some(body) => request.json(&body),
            None => request,
        };
        let response = match request
            .timeout(std::time::Duration::from_secs(RAW_REQUEST_TIMEOUT_SECS))
            .send()
            .await
        {
            Ok(response) => response,
            Err(error) => {
                return Err(Box::new(std::io::Error::other(format!(
                    "request failed: {error}"
                ))))
            }
        };
        let status = response.status();
        let text = match response.text().await {
            Ok(text) => text,
            Err(error) => {
                return Err(Box::new(std::io::Error::other(format!(
                    "failed to read response body (HTTP {status}): {error}"
                ))))
            }
        };
        if text.trim().is_empty() {
            if status.is_success() {
                return Ok(None);
            }
            return Err(Box::new(std::io::Error::other(format!(
                "HTTP {status} with an empty response body"
            ))));
        }
        if !status.is_success() {
            return Err(Box::new(std::io::Error::other(format!(
                "HTTP {status}: {}",
                text.chars().take(512).collect::<String>()
            ))));
        }
        match serde_json::from_str::<Value>(&text) {
            Ok(value) => Ok(Some(value)),
            Err(_) => Err(Box::new(std::io::Error::other(format!(
                "API returned non-JSON response (HTTP {status}): {}",
                text.chars().take(512).collect::<String>()
            )))),
        }
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

    async fn delete(&self, url: String) -> ResultWithDefaultError<()> {
        match self.http_client.delete(url).send().await {
            Err(_) => Err(Box::new(ApiError::Network)),
            Ok(response) => {
                if response.status().is_success() {
                    Ok(())
                } else {
                    Err(Box::new(ApiError::Deserialization))
                }
            }
        }
    }
}

#[async_trait]
impl ApiClient for V9ApiClient {
    async fn get_user(&self) -> ResultWithDefaultError<User> {
        let url = format!("{}/me", self.base_url);
        return self.get::<User>(url).await;
    }

    async fn send_raw(
        &self,
        path: &str,
        method: &str,
        body: Option<Value>,
    ) -> ResultWithDefaultError<Option<Value>> {
        self.send_raw_request(path, method, body).await
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

    async fn delete_time_entry(
        &self,
        workspace_id: i64,
        time_entry_id: i64,
    ) -> ResultWithDefaultError<()> {
        let url = format!(
            "{}/workspaces/{}/time_entries/{}",
            self.base_url, workspace_id, time_entry_id
        );
        self.delete(url).await
    }

    async fn create_project(
        &self,
        workspace_id: i64,
        name: String,
        color: String,
    ) -> ResultWithDefaultError<Project> {
        let url = format!("{}/workspaces/{}/projects", self.base_url, workspace_id);
        let body = NetworkCreateProject {
            name,
            workspace_id,
            color,
            is_private: false,
            active: true,
        };
        let network_project = self
            .post::<NetworkProject, NetworkCreateProject>(url, &body)
            .await?;
        Ok(Project {
            id: network_project.id,
            name: network_project.name,
            workspace_id: network_project.workspace_id,
            client: None,
            is_private: network_project.is_private,
            active: network_project.active,
            at: network_project.at,
            created_at: network_project.created_at,
            color: network_project.color,
            billable: network_project.billable,
        })
    }

    async fn delete_project(
        &self,
        workspace_id: i64,
        project_id: i64,
    ) -> ResultWithDefaultError<()> {
        let url = format!(
            "{}/workspaces/{}/projects/{}",
            self.base_url, workspace_id, project_id
        );
        self.delete(url).await
    }

    async fn rename_project(
        &self,
        workspace_id: i64,
        project_id: i64,
        new_name: String,
    ) -> ResultWithDefaultError<Project> {
        let url = format!(
            "{}/workspaces/{}/projects/{}",
            self.base_url, workspace_id, project_id
        );
        let body = NetworkRenameProject { name: new_name };
        let network_project = self
            .put::<NetworkProject, NetworkRenameProject>(url, &body)
            .await?;
        Ok(Project {
            id: network_project.id,
            name: network_project.name,
            workspace_id: network_project.workspace_id,
            client: None,
            is_private: network_project.is_private,
            active: network_project.active,
            at: network_project.at,
            created_at: network_project.created_at,
            color: network_project.color,
            billable: network_project.billable,
        })
    }

    async fn get_tags(&self, workspace_id: i64) -> ResultWithDefaultError<Vec<Tag>> {
        let network_tags = self.get_workspace_tags(workspace_id).await?;
        Ok(network_tags
            .into_iter()
            .map(|t| Tag {
                id: t.id,
                name: t.name,
                workspace_id: t.workspace_id,
            })
            .collect())
    }

    async fn create_tag(&self, workspace_id: i64, name: String) -> ResultWithDefaultError<Tag> {
        let url = format!("{}/workspaces/{}/tags", self.base_url, workspace_id);
        let body = NetworkCreateTag { name, workspace_id };
        let network_tag = self
            .post::<NetworkTag, NetworkCreateTag>(url, &body)
            .await?;
        Ok(Tag {
            id: network_tag.id,
            name: network_tag.name,
            workspace_id: network_tag.workspace_id,
        })
    }

    async fn delete_tag(&self, workspace_id: i64, tag_id: i64) -> ResultWithDefaultError<()> {
        let url = format!(
            "{}/workspaces/{}/tags/{}",
            self.base_url, workspace_id, tag_id
        );
        self.delete(url).await
    }

    async fn rename_tag(
        &self,
        workspace_id: i64,
        tag_id: i64,
        new_name: String,
    ) -> ResultWithDefaultError<Tag> {
        let url = format!(
            "{}/workspaces/{}/tags/{}",
            self.base_url, workspace_id, tag_id
        );
        let body = NetworkRenameTag {
            name: new_name,
            workspace_id,
        };
        let network_tag = self.put::<NetworkTag, NetworkRenameTag>(url, &body).await?;
        Ok(Tag {
            id: network_tag.id,
            name: network_tag.name,
            workspace_id: network_tag.workspace_id,
        })
    }

    async fn get_time_entries_filtered(
        &self,
        since: Option<String>,
        until: Option<String>,
    ) -> ResultWithDefaultError<Vec<TimeEntry>> {
        let (network_entries, network_projects, network_tasks, network_clients) = tokio::join!(
            self.get_time_entries(since.as_deref(), until.as_deref()),
            self.get_projects(),
            self.get_tasks(),
            self.get_clients(),
        );

        let clients: HashMap<i64, crate::models::Client> = network_clients
            .unwrap_or_default()
            .into_iter()
            .map(|c| {
                (
                    c.id,
                    crate::models::Client {
                        id: c.id,
                        name: c.name,
                        workspace_id: c.wid,
                    },
                )
            })
            .collect();

        let projects: HashMap<i64, Project> = network_projects
            .unwrap_or_default()
            .into_iter()
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
                        color: p.color,
                        billable: p.billable,
                    },
                )
            })
            .collect();

        let tasks: HashMap<i64, Task> = network_tasks
            .unwrap_or_default()
            .into_iter()
            .filter_map(|t| {
                projects.get(&t.project_id).map(|project| {
                    (
                        t.id,
                        Task {
                            id: t.id,
                            name: t.name,
                            project: project.clone(),
                            workspace_id: t.workspace_id,
                        },
                    )
                })
            })
            .collect();

        let entries = network_entries
            .unwrap_or_default()
            .into_iter()
            .map(|te| TimeEntry {
                id: te.id,
                description: te.description,
                start: te.start,
                stop: te.stop,
                duration: te.duration,
                billable: te.billable,
                workspace_id: te.workspace_id,
                tags: te.tags.unwrap_or_default(),
                project: projects.get(&te.project_id.unwrap_or(-1)).cloned(),
                task: tasks.get(&te.task_id.unwrap_or(-1)).cloned(),
                ..Default::default()
            })
            .collect();

        Ok(entries)
    }

    async fn get_entities(&self) -> ResultWithDefaultError<Entities> {
        let (
            network_time_entries,
            network_projects,
            network_tasks,
            network_clients,
            network_workspaces,
        ) = tokio::join!(
            self.get_time_entries(None, None),
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
                tags: te.tags.clone().unwrap_or_default(),
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
            tags: Vec::new(),
        })
    }
}

const ALLOWED_RAW_HOSTS: &[&str] = &["api.track.toggl.com"];
const RAW_REQUEST_TIMEOUT_SECS: u64 = 30;

/// Resolves the path argument of the raw API command into a full URL.
/// Relative paths are joined onto the client's base URL and validated;
/// absolute URLs are allowed only when their host is on the allowlist
/// (e.g. the reports API at `https://api.track.toggl.com/reports/api/v3`).
fn resolve_url(base_url: &str, path: &str) -> ResultWithDefaultError<String> {
    if path.starts_with("http://") || path.starts_with("https://") {
        let url = reqwest::Url::parse(path).map_err(|_| -> Box<dyn std::error::Error + Send> {
            Box::new(std::io::Error::other(format!("invalid URL: {path}")))
        })?;
        if !ALLOWED_RAW_HOSTS.contains(&url.host_str().unwrap_or("")) {
            return Err(Box::new(std::io::Error::other(format!(
                "URL host {:?} is not allowed; only {} URLs are supported",
                url.host_str().unwrap_or(""),
                ALLOWED_RAW_HOSTS.join(", ")
            ))));
        }
        return Ok(path.to_string());
    }
    if !path.starts_with('/') || path.starts_with("//") {
        return Err(Box::new(std::io::Error::other(format!(
            "invalid API path \"{path}\": expected a path starting with a single '/', \
             e.g. \"/me/time_entries/current\""
        ))));
    }
    let joined = format!("{base_url}{path}");
    reqwest::Url::parse(&joined).map_err(|_| -> Box<dyn std::error::Error + Send> {
        Box::new(std::io::Error::other(format!(
            "invalid API path \"{path}\": not a valid URL"
        )))
    })?;
    Ok(joined)
}

#[cfg(test)]
mod raw_url_tests {
    use super::resolve_url;
    use tokio_test::assert_err;

    const BASE_URL: &str = "https://track.toggl.com/api/v9";

    #[test]
    fn joins_relative_path() {
        assert_eq!(
            resolve_url(BASE_URL, "/me/time_entries/current").unwrap(),
            format!("{BASE_URL}/me/time_entries/current")
        );
    }

    #[test]
    fn rejects_path_without_leading_slash() {
        assert_err!(resolve_url(BASE_URL, "me/time_entries"));
    }

    #[test]
    fn rejects_double_slash_path() {
        assert_err!(resolve_url(BASE_URL, "//evil.com/me"));
    }

    #[test]
    fn allows_allowed_host_url() {
        assert_eq!(
            resolve_url(
                BASE_URL,
                "https://api.track.toggl.com/reports/api/v3/search/time_entries"
            )
            .unwrap(),
            "https://api.track.toggl.com/reports/api/v3/search/time_entries"
        );
    }

    #[test]
    fn rejects_other_host_url() {
        assert_err!(resolve_url(BASE_URL, "https://example.com/me"));
    }

    #[test]
    fn percent_encodes_odd_characters_in_relative_paths() {
        // The url crate percent-encodes instead of rejecting; reqwest applies
        // the same encoding when the path is joined for the request.
        let url = resolve_url(BASE_URL, "/me/time entries").unwrap();
        assert!(url.ends_with("/me/time entries"));
    }
}
