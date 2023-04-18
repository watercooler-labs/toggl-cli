use std::fs;

use colored::Colorize;

use crate::{error::ConfigError, models::ResultWithDefaultError, utilities};

pub struct ConfigManageCommand;

impl ConfigManageCommand {
    pub async fn execute(delete: bool, edit: bool, show_path: bool) -> ResultWithDefaultError<()> {
        let path = super::locate::locate_config_path()?;

        if delete {
            let path = path.as_path();
            return fs::remove_file(path).map_err(|e| e.into()).map(|_| {
                println!(
                    "{} {}",
                    "Config file deleted from".red().bold(),
                    path.display()
                );
            });
        }
        if edit {
            return utilities::open_path_in_editor(path);
        }
        if show_path {
            println!("{}", path.display());
            return Ok(());
        }
        match super::parser::get_config_from_file(path) {
            Ok(config) => {
                println!("{}", config);
                Ok(())
            }
            Err(_) => Err(Box::new(ConfigError::Parse)),
        }
    }
}
