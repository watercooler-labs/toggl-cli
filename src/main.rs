mod api;
mod arguments;
mod commands;
mod constants;
mod credentials;
mod error;
mod models;
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
use colored::Colorize;
use commands::auth::AuthenticationCommand;
use commands::cont::ContinueCommand;
use commands::list::ListCommand;
use commands::running::RunningTimeEntryCommand;
use commands::stop::StopCommand;
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
    match command {
        None => RunningTimeEntryCommand::execute(get_api_client()?).await?,
        Some(subcommand) => match subcommand {
            Stop => StopCommand::execute(get_api_client()?).await?,
            Continue => ContinueCommand::execute(get_api_client()?).await?,
            List { number } => ListCommand::execute(get_api_client()?, number).await?,
            Current | Running => RunningTimeEntryCommand::execute(get_api_client()?).await?,
            Start {
                description: _,
                project: _,
            } => (),
            Auth { api_token } => {
                let credentials = Credentials { api_token };
                let api_client = V9ApiClient::from_credentials(credentials)?;
                AuthenticationCommand::execute(api_client, get_storage()).await?
            }
        },
    }

    Ok(())
}

fn get_api_client() -> ResultWithDefaultError<impl ApiClient> {
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
            Err(err)
        }
    };
}

fn get_storage() -> impl CredentialsStorage {
    let keyring = Keyring::new("togglcli", "default");
    KeyringStorage::new(keyring)
}
