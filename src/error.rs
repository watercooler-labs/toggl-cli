use crate::constants;
use colored::Colorize;
use std::error::Error;
use std::fmt::Display;

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
