use crate::constants;
use colored::Colorize;
use std::error::Error;
use std::fmt::Display;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ApiError {
    Network,
    Deserialization,
}

impl Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = match self {
            ApiError::Network => format!("{}", constants::NETWORK_ERROR_MESSAGE.red()),
            ApiError::Deserialization => format!(
                "{}\n{} {}",
                constants::DESERIALIZATION_ERROR_MESSAGE.red(),
                constants::OUTDATED_APP_ERROR_MESSAGE.blue().bold(),
                constants::ISSUE_LINK.blue().bold().underline()
            ),
        };
        write!(f, "{}", summary)
    }
}

impl Error for ApiError {}

#[derive(Debug)]
pub enum StorageError {
    Write,
}

impl Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = format!(
            "{}\n{} {}",
            constants::CREDENTIALS_ACCESS_ERROR.red(),
            constants::OUTDATED_APP_ERROR_MESSAGE.blue().bold(),
            constants::ISSUE_LINK.blue().bold().underline()
        );
        write!(f, "{}", summary)
    }
}

impl Error for StorageError {}

#[derive(Debug)]
pub enum PickerError {
    Cancelled,
    FzfNotInstalled,
    Generic,
}

impl Display for PickerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = format!(
            "{}",
            match self {
                PickerError::Cancelled => constants::OPERATION_CANCELLED,
                PickerError::FzfNotInstalled => constants::FZF_NOT_INSTALLED_ERROR,
                PickerError::Generic => constants::GENERIC_ERROR,
            }
            .red(),
        );
        write!(f, "{}", summary)
    }
}

impl Error for PickerError {}

#[derive(Debug)]
pub enum ConfigError {
    Parse,
    FileNotFound,
    UnrecognizedMarco(String),
    ShellResolution(String, String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = match self {
            ConfigError::Parse => {
                format!(
                    "{}\nTo edit the configuration file in your editor run {}",
                    constants::CONFIG_PARSE_ERROR.red().bold(),
                    "toggl config --edit".blue().bold(),
                )
            }
            ConfigError::FileNotFound => {
                format!(
                    "{}\nRun {} to create one",
                    constants::CONFIG_FILE_NOT_FOUND_ERROR.red().bold(),
                    "toggl config init".blue().bold(),
                )
            }
            ConfigError::UnrecognizedMarco(marco) => {
                format!(
                    "{}: {}",
                    constants::CONFIG_UNRECOGNIZED_MACRO_ERROR.red().bold(),
                    marco.red().bold(),
                )
            }
            ConfigError::ShellResolution(command, output_or_error) => {
                format!(
                    "{}: {}\n{}: {}",
                    constants::CONFIG_SHELL_MACRO_RESOLUTION_ERROR.red(),
                    output_or_error.red().bold(),
                    "Command".yellow(),
                    command.yellow().bold(),
                )
            }
        };
        writeln!(f, "{}", summary)
    }
}

impl Error for ConfigError {}

#[derive(Debug)]
pub enum CliError {
    DirectoryNotFound(PathBuf),
    NotADirectory(PathBuf),
}

impl Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let summary = match self {
            CliError::DirectoryNotFound(path) => {
                format!(
                    "{}: {}",
                    constants::DIRECTORY_NOT_FOUND_ERROR.red(),
                    path.display()
                )
            }
            CliError::NotADirectory(path) => {
                format!(
                    "{}: {}",
                    constants::NOT_A_DIRECTORY_ERROR.red(),
                    path.display()
                )
            }
        };
        writeln!(f, "{}", summary)
    }
}

impl Error for CliError {}
