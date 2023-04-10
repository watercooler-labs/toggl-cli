use colored::Colorize;

use crate::{models::ResultWithDefaultError, utilities};

pub struct ConfigInitCommand;

impl ConfigInitCommand {
    pub async fn execute(edit: bool) -> ResultWithDefaultError<()> {
        let config_path = super::locate::locate_config_path();
        match config_path {
            Ok(path) => {
                if edit {
                    utilities::open_path_in_editor(&path)?
                } else {
                    println!(
                        "{} {}",
                        "Config file already exists at".yellow(),
                        path.display()
                    );
                    return Ok(());
                }
            }
            Err(_) => {
                let default_config = include_bytes!("./fixtures/default.toml");
                let config_path = super::locate::get_config_path_for_current_dir()?;
                let msg = format!(
                    "{} {}",
                    "Created config at".green().bold(),
                    config_path.display()
                );
                std::fs::write(config_path, default_config)?;
                println!("{}", msg);
            }
        }
        Ok(())
    }
}
