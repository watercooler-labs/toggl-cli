use crate::error;
use crate::models;
use async_trait::async_trait;
use error::StorageError;
use keyring::Entry;
#[cfg(test)]
use mockall::automock;
use models::ResultWithDefaultError;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const DEFAULT_PROFILE: &str = "default";

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

    let username = profile.unwrap_or(DEFAULT_PROFILE);
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
    read_profiles(&get_profiles_path())
}

fn read_profiles(path: &Path) -> Vec<String> {
    let content = match fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return vec![],
        Err(err) => {
            eprintln!("Warning: failed to read profiles: {err}");
            return vec![];
        }
    };
    serde_json::from_str(&content).unwrap_or_else(|err| {
        eprintln!("Warning: failed to parse profiles: {err}");
        vec![]
    })
}

fn write_profiles(path: &Path, profiles: &[String]) -> ResultWithDefaultError<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send>)?;
    }
    let content = serde_json::to_string_pretty(profiles)
        .map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send>)?;
    fs::write(path, content).map_err(|err| Box::new(err) as Box<dyn std::error::Error + Send>)?;
    Ok(())
}

pub fn add_profile(name: &str) -> ResultWithDefaultError<()> {
    add_profile_at(&get_profiles_path(), name)
}

fn add_profile_at(path: &Path, name: &str) -> ResultWithDefaultError<()> {
    let mut profiles = read_profiles(path);
    if profiles.iter().any(|profile| profile == name) {
        return Ok(());
    }
    profiles.push(name.to_string());
    profiles.sort();
    profiles.dedup();
    write_profiles(path, &profiles)
}

pub fn remove_profile(name: &str) -> ResultWithDefaultError<()> {
    remove_profile_at(&get_profiles_path(), name)
}

fn remove_profile_at(path: &Path, name: &str) -> ResultWithDefaultError<()> {
    let mut profiles = read_profiles(path);
    let before = profiles.len();
    profiles.retain(|profile| profile != name);
    if profiles.len() == before {
        return Ok(());
    }
    write_profiles(path, &profiles)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_profiles_path() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir.path().join("profiles.json");
        (dir, path)
    }

    #[test]
    fn missing_file_reads_as_empty() {
        let (_dir, path) = temp_profiles_path();
        assert_eq!(read_profiles(&path), Vec::<String>::new());
    }

    #[test]
    fn add_profile_creates_sorted_file() {
        let (_dir, path) = temp_profiles_path();
        add_profile_at(&path, "work").expect("add work");
        add_profile_at(&path, "default").expect("add default");
        add_profile_at(&path, "personal").expect("add personal");
        assert_eq!(
            read_profiles(&path),
            vec!["default", "personal", "work"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<String>>()
        );
    }

    #[test]
    fn add_profile_is_idempotent() {
        let (_dir, path) = temp_profiles_path();
        add_profile_at(&path, "work").expect("add work");
        add_profile_at(&path, "work").expect("add work again");
        assert_eq!(read_profiles(&path), vec!["work".to_string()]);
    }

    #[test]
    fn remove_profile_takes_out_entry() {
        let (_dir, path) = temp_profiles_path();
        add_profile_at(&path, "work").expect("add work");
        remove_profile_at(&path, "work").expect("remove work");
        assert!(read_profiles(&path).is_empty());
    }

    #[test]
    fn removing_unknown_profile_is_a_noop() {
        let (_dir, path) = temp_profiles_path();
        remove_profile_at(&path, "ghost").expect("remove unknown profile");
        assert!(!path.exists());
    }
}
