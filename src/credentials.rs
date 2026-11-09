use crate::error;
use crate::models;
use async_trait::async_trait;
use error::StorageError;
use keyring::Entry;
#[cfg(test)]
use mockall::automock;
use models::ResultWithDefaultError;
use std::fs;
use std::path::PathBuf;

pub struct Credentials {
    pub api_token: String,
}

#[cfg_attr(test, automock)]
#[async_trait]
pub trait CredentialsStorage {
    fn read(&self) -> ResultWithDefaultError<Credentials>;
    fn persist(&self, api_token: String) -> ResultWithDefaultError<()>;
    fn clear(&self) -> ResultWithDefaultError<()>;
}

pub struct KeyringStorage {
    keyring: Entry,
}

impl KeyringStorage {
    pub fn new(keyring: Entry) -> KeyringStorage {
        Self { keyring }
    }
}

impl CredentialsStorage for KeyringStorage {
    fn read(&self) -> ResultWithDefaultError<Credentials> {
        self.keyring
            .get_password()
            .map(|api_token| Credentials { api_token })
            .map_err(|keyring_err| match keyring_err {
                keyring::Error::NoEntry => Box::new(StorageError::Read),
                _ => Box::new(StorageError::Unknown) as Box<dyn std::error::Error + Send>,
            })
    }

    fn persist(&self, api_token: String) -> ResultWithDefaultError<()> {
        match self.keyring.set_password(api_token.as_str()) {
            Err(keyring_err) => {
                eprintln!("Error writing to keyring: {keyring_err}");
                Err(Box::new(StorageError::Write))
            }
            Ok(_) => Ok(()),
        }
    }

    fn clear(&self) -> ResultWithDefaultError<()> {
        match self.keyring.delete_password() {
            Err(keyring_err) => {
                eprintln!("Error deleting from keyring: {keyring_err}");
                Err(Box::new(StorageError::Delete))
            }
            Ok(_) => Ok(()),
        }
    }
}

pub struct EnvironmentStorage {
    token: String,
}

impl EnvironmentStorage {
    pub fn new(token: String) -> EnvironmentStorage {
        Self { token }
    }
}

impl CredentialsStorage for EnvironmentStorage {
    fn read(&self) -> ResultWithDefaultError<Credentials> {
        Ok(Credentials {
            api_token: self.token.clone(),
        })
    }
    fn persist(&self, _api_token: String) -> ResultWithDefaultError<()> {
        Err(Box::new(StorageError::EnvironmentOverride))
    }
    fn clear(&self) -> ResultWithDefaultError<()> {
        Err(Box::new(StorageError::EnvironmentOverride))
    }
}

pub fn get_storage(profile: Option<&str>) -> Box<dyn CredentialsStorage> {
    if profile.is_none() {
        if let Ok(api_token) = std::env::var("TOGGL_API_TOKEN") {
            return Box::new(EnvironmentStorage::new(api_token));
        }
    }

    let username = profile.unwrap_or("default");
    let keyring = Entry::new("togglcli", username)
        .unwrap_or_else(|err| panic!("Couldn't create credentials_storage: {err}"));
    Box::new(KeyringStorage::new(keyring))
}

fn get_profiles_path() -> PathBuf {
    directories::ProjectDirs::from("studio.watercooler", "labs", "toggl-cli")
        .unwrap()
        .config_local_dir()
        .join("profiles.json")
}

pub fn list_profiles() -> Vec<String> {
    let path = get_profiles_path();
    if !path.exists() {
        return vec![];
    }
    
    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|e| {
            eprintln!("Warning: failed to parse profiles.json: {}", e);
            vec![]
        }),
        Err(e) => {
            eprintln!("Warning: failed to read profiles.json: {}", e);
            vec![]
        }
    }
}

pub fn add_profile(name: &str) {
    let mut profiles = list_profiles();
    if !profiles.contains(&name.to_string()) {
        profiles.push(name.to_string());
        profiles.sort();
        let path = get_profiles_path();
        if let Some(parent) = path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Warning: failed to create config directory: {}", e);
                return;
            }
        }
        if let Err(e) = fs::write(&path, serde_json::to_string_pretty(&profiles).unwrap_or_default()) {
            eprintln!("Warning: failed to save profiles: {}", e);
        }
    }
}

pub fn remove_profile(name: &str) {
    let mut profiles = list_profiles();
    profiles.retain(|p| p != name);
    let path = get_profiles_path();
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Warning: failed to create config directory: {}", e);
            return;
        }
    }
    if let Err(e) = fs::write(&path, serde_json::to_string_pretty(&profiles).unwrap_or_default()) {
        eprintln!("Warning: failed to save profiles: {}", e);
    }
}
