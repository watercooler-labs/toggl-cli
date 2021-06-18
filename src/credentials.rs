use keyring::Keyring;

pub struct Credentials { pub api_token: String }

impl Credentials {

    fn get_keyring() -> Keyring<'static> {
        return Keyring::new("togglcli", "default")
    }

    pub fn read() -> Option<Credentials> {
        return match Credentials::get_keyring().get_password() {
            Ok(api_token) => Some(Credentials { api_token: api_token }),
            Err(_) => None,
        }
    }

    pub fn persist(api_token: String) -> Result<(), Box<dyn std::error::Error>> {
        Credentials::get_keyring().set_password(api_token.as_str())?;
        return Ok(());
    }
}