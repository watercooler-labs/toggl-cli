use crate::error;
use crate::models;
use async_trait::async_trait;
use error::StorageError;
use keyring::Keyring;
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
}

pub struct KeyringStorage {
    keyring: Keyring<'static>,
}

impl KeyringStorage {
    pub fn new(keyring: Keyring<'static>) -> KeyringStorage {
        Self { keyring }
    }
}

impl CredentialsStorage for KeyringStorage {
    fn read(&self) -> ResultWithDefaultError<Credentials> {
        let api_token = self.keyring.get_password()?;
        Ok(Credentials { api_token })
    }

    fn persist(&self, api_token: String) -> ResultWithDefaultError<()> {
        match self.keyring.set_password(api_token.as_str()) {
            Err(_) => Err(Box::new(StorageError::Write)),
            Ok(_) => Ok(()),
        }
    }
}
