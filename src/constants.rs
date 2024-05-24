pub const OUTDATED_APP_ERROR_MESSAGE: &str =
    "Make sure you are on the latest version of the app or file an issue here:";
pub const CLIENT_NAME: &str = "github.com/watercooler-labs/toggl-cli/toggl-cli";
pub const GENERIC_ERROR: &str = "Something went wrong.";
pub const NETWORK_ERROR_MESSAGE: &str =
    "An error occurred when making a network request\nCheck your connection and try again.";
pub const DESERIALIZATION_ERROR_MESSAGE: &str = "An error occurred when making a network request.";
pub const ISSUE_LINK: &str = "https://github.com/watercooler-labs/toggl-cli/issues/new";
pub const CREDENTIALS_ACCESS_ERROR: &str =
    "An unknown error occurred while reading your credentials.";
pub const CREDENTIALS_WRITE_ERROR: &str =
    "An unknown error occurred while writing your credentials.";
pub const CREDENTIALS_DELETE_ERROR: &str =
    "An unknown error occurred while deleting your credentials.";
pub const CREDENTIALS_EMPTY_ERROR: &str =
    "Please set your API token first by calling toggl auth <API_TOKEN>.";
pub const CREDENTIALS_FIND_TOKEN_MESSAGE: &str = "You can find your API token at";
pub const CREDENTIALS_FIND_TOKEN_LINK: &str = "https://track.toggl.com/profile";

pub const FZF_NOT_INSTALLED_ERROR: &str = "fzf could not be found. Is it installed?";
pub const OPERATION_CANCELLED: &str = "Operation cancelled";
pub const CONFIG_FILE_NOT_FOUND_ERROR: &str = "No config file found";
pub const CONFIG_PARSE_ERROR: &str = "Failed to parse config file";
pub const CONFIG_UNRECOGNIZED_MACRO_ERROR: &str = "Unrecognized macro in config file";
pub const CONFIG_SHELL_MACRO_RESOLUTION_ERROR: &str = "Failed to resolve shell macro";
pub const NO_PROJECT: &str = "No Project";
pub const NO_DESCRIPTION: &str = "(no description)";
pub const DIRECTORY_NOT_FOUND_ERROR: &str = "Directory not found";
pub const NOT_A_DIRECTORY_ERROR: &str = "Not a directory";

#[cfg(target_os = "macos")]
pub const SIMPLE_HOME_PATH: &str = "~/Library/Application Support";
#[cfg(windows)]
pub const SIMPLE_HOME_PATH: &str = "%localappdata%";
#[cfg(target_os = "linux")]
pub const SIMPLE_HOME_PATH: &str = "~/.config";
