use std::fs;

use colored::Colorize;

use crate::{models::ResultWithDefaultError, utilities};

pub struct ConfigManageCommand;

impl ConfigManageCommand {
    pub async fn execute(delete: bool, edit: bool, show_path: bool) -> ResultWithDefaultError<()> {
        let path = super::locate::locate_config_path().map_err(
            |e|
            {
                println!("{}", "No config file found".red().bold());
                println!("Run {} to create one", "toggl config init".blue().bold());
                e
            }
        )?;

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
            }
            Err(_) => {
                println!("{}", "No config file found".red().bold());
                println!("Run {} to create one", "toggl config init".blue().bold());
            }
        }
        Ok(())
    }
}
