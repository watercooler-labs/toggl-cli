use crate::models;
use async_trait::async_trait;
use keyring::Keyring;
use models::ResultWithDefaultError;

pub struct Credentials {
    pub api_token: String,
}

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
        self.keyring.set_password(api_token.as_str())?;
        Ok(())
    }
}
