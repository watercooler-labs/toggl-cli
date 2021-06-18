mod arguments;
mod credentials;
use arguments::CommandLineArguments;
use arguments::Command;
use arguments::Command::Auth;
use arguments::Command::Current;
use arguments::Command::Running;
use arguments::Command::Stop;
use arguments::Command::Start;
use credentials::Credentials;
use structopt::StructOpt;

fn main() {
    match Credentials::from_file() {
        Invalid => ask_for_auth(),
        Valid(credentials) => {
            let parsed_args = CommandLineArguments::from_args();
            execute_subcommand(credentials, parsed_args.cmd);
        }
    }
}

pub fn execute_subcommand(_credentials: ValidCredentials, command: Option<Command>) {
    match command {
        None => (),
        Some(subcommand) => match subcommand {
            Current => (),
            Running => (),
            Stop => (),
            Start { description: _, project: _ } => (),
            Auth { api_token: _ } => ()
        }
    };
}

fn ask_for_auth() {
    println!("Please set your API token first by calling toggl auth <API_TOKEN>.");
    println!("You can find your API token at https://track.toggl.com/profile.");
}