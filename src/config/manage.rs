use std::fs;

use colored::Colorize;

use crate::{models::ResultWithDefaultError, utilities};

pub struct ConfigManageCommand;

impl ConfigManageCommand {
    pub async fn execute(delete: bool, edit: bool, show_path: bool) -> ResultWithDefaultError<()> {
        match (delete, edit, show_path) {
            (true, _, _) => {
                let path = super::locate::locate_config_path()?;
                let path = path.as_path();
                fs::remove_file(path).map_err(|e| e.into()).map(|_| {
                    println!(
                        "{} {}",
                        "Config file deleted from".red().bold(),
                        path.display()
                    );
                })
            }
            (_, true, _) => {
                let path = super::locate::locate_config_path()?;
                utilities::open_path_in_editor(path)
            }
            (_, _, true) => {
                let path = super::locate::locate_config_path()?;
                println!("{}", path.display());
                Ok(())
            }
            _ => {
                match super::locate::locate_config() {
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
    }
}
