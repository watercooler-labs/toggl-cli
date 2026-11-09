use crate::api::client::{ApiClient, V9ApiClient};
use crate::credentials;
use crate::credentials::DEFAULT_PROFILE;
use crate::models::ResultWithDefaultError;
use colored::Colorize;
use tokio::task::JoinHandle;

pub struct ProfilesListCommand;

impl ProfilesListCommand {
    pub async fn execute(
        active_profile: Option<&str>,
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

        for (profile_name, handle) in handles {
            let email = handle.await.unwrap_or(None);
            let is_active = profile_name == active;
            let marker = if is_active { "*" } else { " " };
            let status = if is_active {
                " (active)".blue().to_string()
            } else {
                String::new()
            };
            let user_info = match email {
                Some(email) => email.green().to_string(),
                None => "(not authenticated)".yellow().to_string(),
            };
            println!("{marker} {profile_name:<10} {user_info}{status}");
        }

        Ok(())
    }
}
