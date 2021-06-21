mod api;
mod arguments;
mod credentials;
mod models;
mod commands;
mod constants;
use api::{ApiClient, V9ApiClient};
use arguments::Command;
use arguments::Command::Auth;
use arguments::Command::Continue;
use arguments::Command::Current;
use arguments::Command::List;
use arguments::Command::Running;
use arguments::Command::Start;
use arguments::Command::Stop;
use arguments::CommandLineArguments;
use commands::auth::AuthenticationCommand;
use commands::cont::ContinueCommand;
use commands::list::ListCommand;
use commands::stop::StopCommand;
use commands::running::RunningTimeEntryCommand;
use colored::Colorize;
use credentials::{Credentials, CredentialsStorage, KeyringStorage};
use keyring::Keyring;
use models::ResultWithDefaultError;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> ResultWithDefaultError<()> {
    let parsed_args = CommandLineArguments::from_args();
    return execute_subcommand(parsed_args.cmd).await;
}

pub async fn execute_subcommand(command: Option<Command>) -> ResultWithDefaultError<()> {
    let credentials_storage = get_storage();
    match command {
        None => RunningTimeEntryCommand::execute(ensure_authentication()?).await?,
        Some(subcommand) => match subcommand {
            Current | Running => RunningTimeEntryCommand::execute(ensure_authentication()?).await?,
            Stop => StopCommand::execute(ensure_authentication()?).await?,
            Start {
                description: _,
                project: _,
            } => (),
            Continue => ContinueCommand::execute(ensure_authentication()?).await?,
            Auth { api_token } => {
                let credentials = Credentials { api_token };
                let api_client = V9ApiClient::from_credentials(credentials)?;
                AuthenticationCommand::execute(api_client, credentials_storage).await?
            },
            List { number } => ListCommand::execute(ensure_authentication()?, number).await?,
        },
    }

    Ok(())
}

fn ensure_authentication() -> ResultWithDefaultError<impl ApiClient> {
    let credentials_storage = get_storage();
    return match credentials_storage.read() {
        Ok(credentials) => V9ApiClient::from_credentials(credentials),
        Err(err) => {
            println!(
                "{}\n{} {}",
                "Please set your API token first by calling toggl auth <API_TOKEN>.".red(),
                "You can find your API token at".blue().bold(),
                "https://track.toggl.com/profile".blue().bold().underline()
            );
            return Err(err);
        }
    };
}

fn get_storage() -> impl CredentialsStorage {
    let keyring = Keyring::new("togglcli", "default");
    KeyringStorage::new(keyring)
}
