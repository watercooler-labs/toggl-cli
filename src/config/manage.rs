use std::fs;

use colored::Colorize;

use crate::{error::ConfigError, models::ResultWithDefaultError, utilities};

pub struct ConfigManageCommand;

impl ConfigManageCommand {
    pub async fn execute(delete: bool, edit: bool, show_path: bool) -> ResultWithDefaultError<()> {
        let path = super::locate::locate_config_path()?;
        let display_path = utilities::simplify_config_path_for_display(path.as_path());

        if delete {
            return {
                fs::remove_file(path)
                    .map(|_| {
                        println!(
                            "{} {}",
                            "Config file deleted from".red().bold(),
                            display_path
                        );
                    })
                    .expect("failed to delete config");
                Ok(())
            };
        }
        if edit {
            return utilities::open_path_in_editor(path);
        }
        if show_path {
            println!("{}", display_path);
            return Ok(());
        }
        match super::parser::get_config_from_file(path) {
            Ok(config) => {
                println!("{}", config);
                Ok(())
            }
            Err(e) => {
                eprintln!("In config parse {}", e);
                Err(Box::new(ConfigError::Parse))
            }
        }
    }
}
