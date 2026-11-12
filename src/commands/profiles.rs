use crate::api::client::{ApiClient, V9ApiClient};
use crate::credentials;
use crate::credentials::DEFAULT_PROFILE;
use crate::models::ResultWithDefaultError;
use colored::Colorize;
use serde::Serialize;
use tokio::task::JoinHandle;

#[derive(Serialize)]
struct ProfileInfo {
    name: String,
    email: Option<String>,
    active: bool,
}

pub struct ProfilesListCommand;

impl ProfilesListCommand {
    pub async fn execute(
        active_profile: Option<&str>,
        json_flag: bool,
        proxy: Option<String>,
    ) -> ResultWithDefaultError<()> {
        // The `default` profile may exist in the keyring without being
        // registered in profiles.json (e.g. auth done before profiles
        // existed), so always include it.
        let mut profiles = credentials::list_profiles();
        if !profiles.iter().any(|profile| profile == DEFAULT_PROFILE) {
            profiles.push(DEFAULT_PROFILE.to_string());
            profiles.sort();
        }

        // Without an explicit profile the CLI falls back to `default`,
        // as resolved in main.rs.
        let active = active_profile.unwrap_or(DEFAULT_PROFILE);
        let mut handles: Vec<(String, JoinHandle<Option<String>>)> = vec![];
        for profile_name in profiles {
            let client = credentials::get_storage(Some(&profile_name))
                .read()
                .ok()
                .and_then(|creds| V9ApiClient::from_credentials(creds, proxy.clone()).ok());
            if let Some(client) = client {
                let handle =
                    tokio::spawn(
                        async move { client.get_user().await.ok().map(|user| user.email) },
                    );
                handles.push((profile_name, handle));
            }
        }

        let mut entries: Vec<ProfileInfo> = vec![];
        for (profile_name, handle) in handles {
            let email = handle.await.unwrap_or(None);
            entries.push(ProfileInfo {
                active: profile_name == active,
                email,
                name: profile_name,
            });
        }

        if json_flag {
            // Matches the JSON printing convention of the other list
            // entities in list.rs.
            let json_string = serde_json::to_string_pretty(&entries)
                .expect("failed to serialize profiles to JSON");
            println!("{json_string}");
            return Ok(());
        }

        for entry in entries {
            let marker = if entry.active { "*" } else { " " };
            let status = if entry.active {
                " (active)".blue().to_string()
            } else {
                String::new()
            };
            let user_info = match entry.email {
                Some(email) => email.green().to_string(),
                None => "(not authenticated)".yellow().to_string(),
            };
            println!("{marker} {:<10} {user_info}{status}", entry.name);
        }

        Ok(())
    }
}
