use crate::error;
use crate::models;
use async_trait::async_trait;
use error::StorageError;
use keyring::Entry;
#[cfg(test)]
use mockall::automock;
use models::ResultWithDefaultError;

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
                eprintln!("Error writing to keyring: {}", keyring_err);
                Err(Box::new(StorageError::Write))
            }
            Ok(_) => Ok(()),
        }
    }

    fn clear(&self) -> ResultWithDefaultError<()> {
        match self.keyring.delete_password() {
            Err(keyring_err) => {
                eprintln!("Error deleting from keyring: {}", keyring_err);
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

pub fn get_storage() -> Box<dyn CredentialsStorage> {
    if let Ok(api_token) = std::env::var("TOGGL_API_TOKEN") {
        return Box::new(EnvironmentStorage::new(api_token));
    }

    let keyring = Entry::new("togglcli", "default")
        .unwrap_or_else(|err| panic!("Couldn't create credentials_storage: {err}"));
    Box::new(KeyringStorage::new(keyring))
}
