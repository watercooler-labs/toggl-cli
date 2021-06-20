use crate::models;
use keyring::Keyring;
use models::ResultWithDefaultError;

pub struct Credentials {
    pub api_token: String,
}

impl Credentials {
    fn get_keyring() -> Keyring<'static> {
        Keyring::new("togglcli", "default")
    }

    pub fn read() -> ResultWithDefaultError<Credentials> {
        let api_token = Credentials::get_keyring().get_password()?;
        Ok(Credentials { api_token })
    }

    pub fn persist(api_token: String) -> ResultWithDefaultError<()> {
        Credentials::get_keyring().set_password(api_token.as_str())?;
        Ok(())
    }
}
