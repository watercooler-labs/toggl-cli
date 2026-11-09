mod api;
mod arguments;
mod commands;
mod config;
mod constants;
mod credentials;
mod error;
mod models;
mod parcel;
mod picker;
mod utilities;

use api::client::ApiClient;
use api::client::V9ApiClient;
use arguments::Command::Auth;
use arguments::Command::Config;
use arguments::Command::Continue;
use arguments::Command::CreateProject;
use arguments::Command::CreateTag;
use arguments::Command::Current;
use arguments::Command::Delete;
use arguments::Command::DeleteProject;
use arguments::Command::DeleteTag;
use arguments::Command::Edit;
use arguments::Command::List;
use arguments::Command::Logout;
use arguments::Command::RenameProject;
use arguments::Command::RenameTag;
use arguments::Command::Running;
use arguments::Command::Start;
use arguments::Command::Stop;
use arguments::CommandLineArguments;
use arguments::ConfigSubCommand;
use commands::auth::AuthenticationCommand;
use commands::cont::ContinueCommand;
use commands::create_project::CreateProjectCommand;
use commands::create_tag::CreateTagCommand;
use commands::delete::DeleteCommand;
use commands::delete_project::DeleteProjectCommand;
use commands::delete_tag::DeleteTagCommand;
use commands::edit::EditCommand;
use commands::list::ListCommand;
use commands::profiles::ProfilesListCommand;
use commands::rename_project::RenameProjectCommand;
use commands::rename_tag::RenameTagCommand;
use commands::running::RunningTimeEntryCommand;
use commands::start::StartCommand;
use commands::stop::{StopCommand, StopCommandOrigin};
use credentials::get_storage;
use credentials::Credentials;
use models::ResultWithDefaultError;
use std::io;
use structopt::StructOpt;

#[tokio::main]
async fn main() -> ResultWithDefaultError<()> {
    let parsed_args = CommandLineArguments::from_args();
    match execute_subcommand(parsed_args).await {
        Ok(()) => Ok(()),
        Err(error) => {
            // We are catching the error and pretty printing it instead of letting the
            // program error. Since we are not meant to be used other programs, I think
            // it's fine to always return a 0 error code, but we might wanna revisit this.
            print!("{error}");
            Ok(())
        }
    }
}

async fn execute_subcommand(args: CommandLineArguments) -> ResultWithDefaultError<()> {
    let command = args.cmd;
    let picker = picker::get_picker(args.fzf);
    if let Some(directory) = args.directory {
        if !directory.exists() {
            return Err(Box::new(error::ArgumentError::DirectoryNotFound(directory)));
        }
        if !directory.is_dir() {
            return Err(Box::new(error::ArgumentError::NotADirectory(directory)));
        }
        std::env::set_current_dir(directory).expect("Couldn't set current directory");
    }
    let resolved_profile = resolve_profile(args.profile.as_deref());
    let get_default_api_client = || get_api_client(args.proxy.clone(), resolved_profile.as_deref());
    match command {
        None => RunningTimeEntryCommand::execute(get_default_api_client()?).await?,
        Some(subcommand) => match subcommand {
            Stop => {
                StopCommand::execute(&get_default_api_client()?, StopCommandOrigin::CommandLine)
                    .await?;
            }

            Continue { interactive } => {
                let picker = if interactive { Some(picker) } else { None };
                ContinueCommand::execute(get_default_api_client()?, picker).await?
            }

            List {
                number,
                json,
                since,
                until,
                entity,
            } => {
                if matches!(entity, Some(arguments::Entity::Profile)) {
                    ProfilesListCommand::execute(resolved_profile.as_deref(), args.proxy.clone())
                        .await?;
                } else {
                    ListCommand::execute(
                        get_default_api_client()?,
                        number,
                        json,
                        since,
                        until,
                        entity,
                    )
                    .await?
                }
            }

            Current | Running => {
                RunningTimeEntryCommand::execute(get_default_api_client()?).await?
            }

            Start {
                interactive,
                billable,
                description,
                project,
                tags,
                task,
            } => {
                StartCommand::execute(
                    get_default_api_client()?,
                    picker,
                    description,
                    project,
                    tags,
                    billable,
                    interactive,
                    task,
                )
                .await?
            }

            CreateProject { name, color } => {
                CreateProjectCommand::execute(get_default_api_client()?, name, color).await?
            }

            DeleteProject { name } => {
                DeleteProjectCommand::execute(get_default_api_client()?, name).await?
            }

            RenameProject { old_name, new_name } => {
                RenameProjectCommand::execute(get_default_api_client()?, old_name, new_name).await?
            }

            CreateTag { name } => {
                CreateTagCommand::execute(get_default_api_client()?, name).await?
            }

            DeleteTag { name } => {
                DeleteTagCommand::execute(get_default_api_client()?, name).await?
            }

            RenameTag { old_name, new_name } => {
                RenameTagCommand::execute(get_default_api_client()?, old_name, new_name).await?
            }

            Edit {
                id,
                description,
                project,
                tags,
                start_time,
                stop_time,
            } => {
                EditCommand::execute(
                    get_default_api_client()?,
                    id,
                    description,
                    project,
                    tags,
                    start_time,
                    stop_time,
                )
                .await?
            }

            Delete { id } => DeleteCommand::execute(get_default_api_client()?, id).await?,

            Auth { api_token } => {
                let api_token = match api_token {
                    Some(token) => token,
                    None => match rpassword::prompt_password("API token: ") {
                        Ok(token) => token,
                        Err(error) => return Err(Box::new(error)),
                    },
                };
                let credentials = Credentials { api_token };
                let api_client = V9ApiClient::from_credentials(credentials, args.proxy)?;
                AuthenticationCommand::execute(
                    io::stdout(),
                    api_client,
                    get_storage(resolved_profile.as_deref()),
                )
                .await?;
                credentials::add_profile(
                    resolved_profile
                        .as_deref()
                        .unwrap_or(credentials::DEFAULT_PROFILE),
                )?
            }

            Logout => {
                let storage = get_storage(resolved_profile.as_deref());
                storage.clear()?;
                let profile_name = resolved_profile
                    .as_deref()
                    .unwrap_or(credentials::DEFAULT_PROFILE);
                credentials::remove_profile(profile_name)?;
                println!("Successfully logged out from profile: {}", profile_name);
            }

            Config {
                delete,
                cmd,
                edit,
                path,
            } => match cmd {
                Some(config_command) => match config_command {
                    ConfigSubCommand::Init => {
                        config::init::ConfigInitCommand::execute(edit).await?;
                    }
                    ConfigSubCommand::Active => {
                        config::active::ConfigActiveCommand::execute().await?;
                    }
                },
                None => config::manage::ConfigManageCommand::execute(delete, edit, path).await?,
            },
        },
    }

    Ok(())
}

fn resolve_profile(cli_profile: Option<&str>) -> Option<String> {
    let env_profile = std::env::var("TOGGL_PROFILE").ok();
    let config_profile = config::locate::locate_config_path()
        .ok()
        .and_then(|path| config::parser::get_config_from_file(path).ok())
        .and_then(|track_config| track_config.get_active_config().ok().cloned())
        .and_then(|branch_config| branch_config.profile);
    resolve_profile_from(cli_profile.map(str::to_string), env_profile, config_profile)
}

fn resolve_profile_from(
    cli_profile: Option<String>,
    env_profile: Option<String>,
    config_profile: Option<String>,
) -> Option<String> {
    [cli_profile, env_profile, config_profile]
        .into_iter()
        .flatten()
        .find(|profile| !profile.trim().is_empty())
        .map(|profile| profile.trim().to_string())
}

fn get_api_client(
    proxy: Option<String>,
    profile: Option<&str>,
) -> ResultWithDefaultError<impl ApiClient> {
    let credentials_storage = get_storage(profile);
    match credentials_storage.read() {
        Ok(credentials) => V9ApiClient::from_credentials(credentials, proxy),
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_profile_takes_priority_over_environment_and_config() {
        assert_eq!(
            resolve_profile_from(
                Some("cli".to_string()),
                Some("env".to_string()),
                Some("config".to_string())
            ),
            Some("cli".to_string())
        );
    }

    #[test]
    fn empty_profile_values_are_ignored() {
        assert_eq!(
            resolve_profile_from(None, Some(String::new()), Some("   ".to_string())),
            None
        );
        assert_eq!(
            resolve_profile_from(None, Some(" work ".to_string()), None),
            Some("work".to_string())
        );
    }
}
