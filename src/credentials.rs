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
