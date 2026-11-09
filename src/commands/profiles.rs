use crate::api::client::{ApiClient, V9ApiClient};
use crate::credentials;
use crate::models::ResultWithDefaultError;
use colored::Colorize;

pub struct ProfilesListCommand;

impl ProfilesListCommand {
    pub async fn execute(
        active_profile: Option<&str>,
        proxy: Option<String>,
    ) -> ResultWithDefaultError<()> {
        let profiles = credentials::list_profiles();
        
        if profiles.is_empty() {
            println!("No profiles configured. Run 'toggl auth --profile <name>' to add one.");
            return Ok(());
        }

        let mut clients: Vec<(String, Option<V9ApiClient>)> = vec![];
        for profile_name in &profiles {
            let storage = credentials::get_storage(Some(profile_name));
            let client = storage
                .read()
                .ok()
                .and_then(|creds| V9ApiClient::from_credentials(creds, proxy.clone()).ok());
            clients.push((profile_name.clone(), client));
        }

        let mut handles = vec![];
        for (profile_name, client) in clients {
            match client {
                Some(client) => {
                    let handle = tokio::spawn(async move {
                        match client.get_user().await {
                            Ok(user) => (profile_name, Some(user.email)),
                            Err(_) => (profile_name, None),
                        }
                    });
                    handles.push(handle);
                }
                None => {
                    handles.push(tokio::spawn(async move { (profile_name, None) }));
                }
            }
        }

        for handle in handles {
            if let Ok((profile_name, email)) = handle.await {
                let is_active = active_profile == Some(profile_name.as_str());
                let marker = if is_active { "* " } else { "  " };
                
                let user_info = match email {
                    Some(e) => e.green().to_string(),
                    None => "(not authenticated or unable to fetch)".yellow().to_string(),
                };
                
                let active_marker = if is_active {
                    " (active)".blue().to_string()
                } else {
                    String::new()
                };
                
                println!("{}{}\t{}{}", marker, profile_name, user_info, active_marker);
            }
        }
        
        Ok(())
    }
}
