mod api;
mod arguments;
mod credentials;
mod models;
use api::ApiClient;
use arguments::Command;
use arguments::Command::Auth;
use arguments::Command::Current;
use arguments::Command::Running;
use arguments::Command::Start;
use arguments::Command::Stop;
use arguments::CommandLineArguments;
use credentials::Credentials;
use models::ResultWithDefaultError;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> ResultWithDefaultError<()> {
    let parsed_args = CommandLineArguments::from_args();
    return execute_subcommand(parsed_args.cmd).await;
}

pub async fn execute_subcommand(command: Option<Command>) -> ResultWithDefaultError<()> {
    match command {
        None => display_running_time_entry().await?,
        Some(subcommand) => match subcommand {
            Current | Running => display_running_time_entry().await?,
            Stop => stop_running_time_entry().await?,
            Start {
                description: _,
                project: _,
            } => (),
            Auth { api_token } => authenticate(api_token).await?,
        },
    }

    Ok(())
}

fn ensure_authentication() -> ResultWithDefaultError<ApiClient> {
    return match Credentials::read() {
        Ok(credentials) => ApiClient::from_credentials(credentials),
        Err(err) => {
            println!("Please set your API token first by calling toggl auth <API_TOKEN>.");
            println!("You can find your API token at https://track.toggl.com/profile.");
            return Err(err);
        }
    };
}

async fn authenticate(api_token: String) -> ResultWithDefaultError<()> {
    let credentials = Credentials {
        api_token: api_token,
    };
    let api_client = ApiClient::from_credentials(credentials)?;
    let user = api_client.get_user().await?;
    let _credentials = Credentials::persist(user.api_token)?;
    println!(
        "Successfully authenticated for user with email {}",
        user.email
    );

    return Ok(());
}

async fn display_running_time_entry() -> ResultWithDefaultError<()> {
    let api_client = ensure_authentication()?;
    match api_client.get_running_time_entry().await? {
        None => println!("No time entry is running at the moment"),
        Some(running_time_entry) => println!("{}", running_time_entry.get_summary()),
    }

    return Ok(());
}

async fn stop_running_time_entry() -> ResultWithDefaultError<()> {
    let api_client = ensure_authentication()?;
    match api_client.get_running_time_entry().await? {
        None => println!("No time entry is running at the moment"),
        Some(running_time_entry) => {
            let _stopped_time_entry = api_client.stop_time_entry(running_time_entry).await?;
            println!("Time entry stopped successfully");
        }
    }

    return Ok(());
}
